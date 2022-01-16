use std::ffi;

use libc;


#[link(name="wfc")]
extern {
    fn wfc_init(wfc: *mut std::ffi::c_void);

    fn wfc_run(wfc: *mut std::ffi::c_void, max_collapse_cnt: libc::c_int) -> libc::c_int;

    fn wfc_export(wfc: *const std::ffi::c_void, filename: *const libc::c_char) -> libc::c_int;

    fn wfc_destroy(wfc: *mut std::ffi::c_void);

    fn wfc_img_copy(image: *const WfcImage) -> *mut WfcImage;

    fn wfc_img_destroy(image: *mut WfcImage);

    fn wfc_output_image(wfc: *mut std::ffi::c_void) -> *mut WfcImage;

    fn wfc_img_load(filename: *const libc::c_char) -> *mut WfcImage;

    fn wfc_img_create(width: libc::c_int, height: libc::c_int, component_cnt: libc::c_int) -> *mut WfcImage;

    fn wfc_overlapping(output_width: libc::c_int,
                       output_height: libc::c_int,
                       image: *mut WfcImage,
                       tile_width: libc::c_int,
                       tile_height: libc::c_int,
                       expand_input: libc::c_int,
                       xflip_tiles: libc::c_int,
                       yflip_tiles: libc::c_int,
                       rotate_tiles: libc::c_int) -> *mut std::ffi::c_void;
}

#[repr(C)]
pub struct WfcImage {
    pub data: *mut i8,
    pub component_cnt: libc::c_int,
    pub width: libc::c_int,
    pub height: libc::c_int,
}

impl WfcImage {
    pub fn new(data: *mut i8,
               component_cnt: libc::c_int,
               width: libc::c_int,
               height: libc::c_int) -> WfcImage {
        return WfcImage { data, component_cnt, width, height };
    }

    pub fn from_vec(width: i32, height: i32, component_cnt: i32, data: Vec<u8>) -> *mut WfcImage {
        unsafe {
            let image_ptr = wfc_img_create(width, height, component_cnt);
            let length = data.len();
            std::ptr::copy_nonoverlapping(data.as_ptr() as *mut u8, (*image_ptr).data as *mut u8, length);
            return image_ptr;
        }
    }

    pub fn from_file(filename: &str) -> Option<*mut WfcImage> {
        unsafe {
            let c_filename = ffi::CString::new(filename).ok()?;
            let image: *mut WfcImage = wfc_img_load(c_filename.as_ptr());
            return Some(image);
        }
    }

    pub fn vec(&self) -> Vec<u8> {
        unsafe {
            let length = self.num_bytes();
            let data: *mut u8 = libc::malloc(length) as *mut u8;
            std::ptr::copy_nonoverlapping(self.data as *mut u8, data, length);

            return Vec::from_raw_parts(data, length, length);
        }
    }

    pub fn num_bytes(&self) -> usize {
        return (self.width * self.height * self.component_cnt) as usize;
    }
}

impl Drop for WfcImage {
    fn drop(&mut self) {
        unsafe {
            wfc_img_destroy(self as *mut WfcImage);
        }
    }
}

/// The main Wfc structure. This structure is normally created
/// by calling overlapping to match the underlying C function wfc_overlapping.
///
/// Once created, the Wfc can be used to create an image with 'run', and this
/// image can be saved with 'export'.
pub struct Wfc {
    pub wfc: *mut libc::c_void,
    pub image: *mut WfcImage,
}

impl Wfc {
    pub fn from_raw_parts(wfc: *mut libc::c_void, image: *mut WfcImage) -> Wfc {
        return Wfc { wfc, image };
    }

    pub fn overlapping(output_width: i32,
                       output_height: i32,
                       image: *mut WfcImage,
                       tile_width: i32,
                       tile_height: i32,
                       expand_input: bool,
                       xflip_tiles: bool,
                       yflip_tiles: bool,
                       rotate_tiles: bool) -> Option<Wfc> {
        unsafe {
            if image.is_null() {
                return None;
            }

            let wfc = wfc_overlapping(output_width,
                                      output_height,
                                      image.as_mut()?,
                                      tile_width,
                                      tile_height,
                                      expand_input as i32,
                                      xflip_tiles as i32,
                                      yflip_tiles as i32,
                                      rotate_tiles as i32);

            if wfc.is_null() {
                return None;
            }

            return Some(Wfc::from_raw_parts(wfc, image.as_mut()?));
        }
    }

    pub fn run(&mut self, max_collapse_cnt: Option<i32>, seed: Option<u32>) -> Result<(), &str> {
        unsafe {
            let mut wfc_ptr = self.wfc.as_mut().ok_or("Wfc pointer invalid")?;
            wfc_init(wfc_ptr);

            // wfc sets the srand seed with time, but only uses rand in wfc_rand.
            // If given a seed, we can apply it between wfc_init and wfc_run.
            if let Some(seed) = seed {
                libc::srand(seed)
            }

            let max_cnt = max_collapse_cnt.unwrap_or(-1);
            let result: libc::c_int = wfc_run(wfc_ptr, max_cnt);

            if result == 0 {
                return Err("wfc_run returned an error!");
            } else {
                return Ok(());
            }
        }
    }

    pub fn export(&mut self, filename: &str) -> Result<(), &str> {
        unsafe {
            let c_filename = ffi::CString::new(filename).map_err(|_| "Filename to CString error")?;
            let mut wfc_ptr = self.wfc.as_mut().ok_or("Wfc pointer invalid")?;
            let result = wfc_export(wfc_ptr, c_filename.as_ptr());

            if result == 0 {
                return Err("wfc_export returned an error!");
            } else {
                return Ok(());
            }
        }
    }

    pub fn output_image(&mut self) -> Option<*mut WfcImage> {
        unsafe {
            return Some(wfc_output_image(self.wfc.as_mut()?));
        };
    }

    /// Convenience function for extracting a copy of the input
    ///
    pub fn vec(&mut self) -> Vec<u8> {
        unsafe {
            // We are assuming that this unwrap succeeds, as this reference
            // is checked when the Wfc is created.
            return self.image.as_ref().unwrap().vec();
        }
    }
}

impl Drop for Wfc {
    fn drop(&mut self) {
        unsafe {
            // segfaults for some reason.
            if let Some(wfc_ptr) = self.wfc.as_mut() {
                wfc_destroy(wfc_ptr);
            }
        }
    }
}

#[test]
pub fn test_overlapping() {
    let image = WfcImage::from_file("data/cave.png").unwrap();
    {
        let wfc = Wfc::overlapping(32, 32, image, 3, 3, true, true, true, true);
    }
    {
        let wfc = Wfc::overlapping(32, 32, image, 3, 3, true, true, true, true);
    }
    {
        let wfc = Wfc::overlapping(32, 32, image, 3, 3, true, true, true, true);
    }
}

#[test]
pub fn test_run() {
    let image = WfcImage::from_file("data/cave.png").unwrap();

    let mut wfc = Wfc::overlapping(32, 32, image, 3, 3, true, true, true, true).unwrap();

    let result = wfc.run(Some(10), Some(1));
    assert_eq!(Ok(()), result);
}

#[test]
pub fn test_export() {
    let image = WfcImage::from_file("data/cave.png").unwrap();

    let mut wfc = Wfc::overlapping(32, 32, image, 3, 3, true, true, true, true).unwrap();
    let result = wfc.run(Some(100), Some(1));
    assert_eq!(Ok(()), result);

    wfc.export("output.png").unwrap();
    std::fs::remove_file("output.png");
}

#[test]
pub fn test_image() {
    let mut image = WfcImage::from_file("data/cave.png").unwrap();

    unsafe {
        let bytes = image.as_ref().unwrap().vec();
        assert_eq!(image.as_ref().unwrap().num_bytes(), bytes.len());
    }
}

