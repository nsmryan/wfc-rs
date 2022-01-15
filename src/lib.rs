use std::ffi;
use std::ptr::NonNull;

use libc;


#[link(name="wfc")]
extern {
    fn wfc_init(wfc: *mut libc::c_void);

    fn wfc_run(wfc: *mut libc::c_void, max_collapse_cnt: libc::c_int) -> libc::c_int;

    fn wfc_export(wfc: *const libc::c_void, filename: *const libc::c_char) -> libc::c_int;

    fn wfc_destroy(wfc: *mut libc::c_void);

    fn wfc_img_copy(image: *const WfcImage) -> *mut WfcImage;

    fn wfc_img_destroy(image: *mut WfcImage);

    fn wfc_output_image(wfc: *mut libc::c_void) -> *mut WfcImage;

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
                       rotate_tiles: libc::c_int) -> *mut libc::c_void;
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

    pub fn from_vec(width: i32, height: i32, component_cnt: i32, data: Vec<u8>) -> Option<NonNull<WfcImage>> {
        unsafe {
            let image_ptr = wfc_img_create(width, height, component_cnt);
            return NonNull::new(image_ptr);
        }
    }

    pub fn from_file(filename: &str) -> Option<NonNull<WfcImage>> {
        unsafe {
            let c_filename = ffi::CString::new(filename).unwrap();
            let image: *mut WfcImage = wfc_img_load(c_filename.as_ptr());
            return NonNull::new(image);
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

/// The main Wfc structure. This structure is normally created
/// by calling overlapping to match the underlying C function wfc_overlapping.
///
/// Once created, the Wfc can be used to create an image with 'run', and this
/// image can be saved with 'export'.
pub struct Wfc {
    pub wfc: NonNull<libc::c_void>,
    pub image: NonNull<WfcImage>,
}

impl Wfc {
    pub fn from_raw_parts(wfc: *mut libc::c_void, image: *mut WfcImage) -> Option<Wfc> {
        let wfc = NonNull::new(wfc)?;
        let image = NonNull::new(image)?;
        return Some(Wfc { wfc, image });
    }

    pub fn overlapping(output_width: i32,
                       output_height: i32,
                       mut image: NonNull<WfcImage>,
                       tile_width: i32,
                       tile_height: i32,
                       expand_input: bool,
                       xflip_tiles: bool,
                       yflip_tiles: bool,
                       rotate_tiles: bool) -> Option<Wfc> {
        unsafe {
            //let mut image = WfcImage::from_file(filename)?;
            let wfc = wfc_overlapping(output_width,
                                      output_height,
                                      image.as_mut(),
                                      tile_width,
                                      tile_height,
                                      expand_input as i32,
                                      xflip_tiles as i32,
                                      yflip_tiles as i32,
                                      rotate_tiles as i32);

            return Wfc::from_raw_parts(wfc, image.as_mut());
        }
    }

    pub fn run(&mut self, max_collapse_cnt: Option<i32>) -> Result<(), &str> {
        unsafe {
            wfc_init(self.wfc.as_mut());

            let max_cnt = max_collapse_cnt.unwrap_or(-1);
            let result: libc::c_int = wfc_run(self.wfc.as_mut(), max_cnt);

            if result == 0 {
                return Err("wfc_run returned an error!");
            } else {
                return Ok(());
            }
        }
    }

    pub fn export(&mut self, filename: &str) -> Result<(), &str> {
        unsafe {
            let c_filename = ffi::CString::new(filename).unwrap();
            let result = wfc_export(self.wfc.as_mut(), c_filename.as_ptr());

            if result == 0 {
                return Err("wfc_export returned an error!");
            } else {
                return Ok(());
            }
        }
    }

    pub fn output_image(&mut self) -> Option<NonNull<WfcImage>> {
        unsafe {
            let image = wfc_output_image(self.wfc.as_mut());
            return NonNull::new(image);
        }
    }

    /// Convenience function for extracting a copy of the input
    ///
    pub fn vec(&mut self) -> Vec<u8> {
        unsafe {
            return self.image.as_ref().vec();
        }
    }
}

impl Drop for Wfc {
    fn drop(&mut self) {
        unsafe {
            // segfaults for some reason.
            //wfc_destroy(self.wfc.as_mut());

            wfc_img_destroy(self.image.as_mut());
        }
    }
}

#[test]
pub fn test_overlapping() {
    let image = WfcImage::from_file("data/cave.png").unwrap();
    let maybe_wfc = Wfc::overlapping(32, 32, image, 3, 3, true, true, true, true);
    assert!(maybe_wfc.is_some());
}

#[test]
pub fn test_run() {
    let image = WfcImage::from_file("data/cave.png").unwrap();
    let maybe_wfc = Wfc::overlapping(32, 32, image, 3, 3, true, true, true, true);

    assert!(maybe_wfc.is_some());

    let mut wfc = maybe_wfc.unwrap();

    let result = wfc.run(Some(1));
    assert_eq!(Ok(()), result);
}

#[test]
pub fn test_export() {
    let image = WfcImage::from_file("data/cave.png").unwrap();
    let maybe_wfc = Wfc::overlapping(32, 32, image, 3, 3, true, true, true, true);
    assert!(maybe_wfc.is_some());

    let mut wfc = maybe_wfc.unwrap();

    let result = wfc.run(Some(100));
    assert_eq!(Ok(()), result);

    wfc.export("output.png").unwrap();
}

#[test]
pub fn test_image() {
    let mut image = WfcImage::from_file("data/cave.png").unwrap();

    unsafe {
        let bytes = image.as_ref().vec();
        assert_eq!(image.as_ref().num_bytes(), bytes.len());
    }
}

