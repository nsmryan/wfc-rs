use std::ffi;
use std::ptr;

use libc;


#[link(name="wfc")]
extern {
    fn wfc_init(wfc: *mut libc::c_void);

    fn wfc_run(wfc: *mut libc::c_void, max_collapse_cnt: libc::c_int) -> libc::c_int;

    fn wfc_export(wfc: *const libc::c_void, filename: *const libc::c_char) -> libc::c_int;

    fn wfc_destroy(wfc: *mut libc::c_void);

    fn wfc_img_destroy(image: *mut libc::c_void);

    // This wrapper uses wfc_export instead.
    //fn wfc_output_image(wfc: *mut libc::c_void) -> *mut libc::c_void;

    fn wfc_img_load(filename: *const libc::c_char) -> *mut libc::c_void;

    // This could be used to receive a Vec<u8> or similar from Rust, with a width, height
    // and component count, and pass directly to WFC. For now, we just support loading
    // from a file, however.
    //fn wfc_img_create(width: libc::c_int, height: libc::c_int, component_cnt: libc::c_int) -> *mut libc::c_void;

    fn wfc_overlapping(output_width: libc::c_int,
                       output_height: libc::c_int,
                       image: *mut libc::c_void,
                       tile_width: libc::c_int,
                       tile_height: libc::c_int,
                       expand_input: libc::c_int,
                       xflip_tiles: libc::c_int,
                       yflip_tiles: libc::c_int,
                       rotate_tiles: libc::c_int) -> *mut libc::c_void;
}

pub struct Wfc {
    wfc: ptr::NonNull<libc::c_void>,
    image: ptr::NonNull<libc::c_void>,
}

impl Wfc {
    pub fn new(wfc: *mut libc::c_void, image: *mut libc::c_void) -> Option<Wfc> {
        let wfc = ptr::NonNull::new(wfc)?;
        let image = ptr::NonNull::new(image)?;
        return Some(Wfc { wfc, image });
    }

    pub fn overlapping(output_width: i32,
                       output_height: i32,
                       filename: &str,
                       tile_width: i32,
                       tile_height: i32,
                       expand_input: i32,
                       xflip_tiles: i32,
                       yflip_tiles: i32,
                       rotate_tiles: i32) -> Option<Wfc> {
        unsafe {
            let c_filename = ffi::CString::new(filename).unwrap();
            let image: *mut libc::c_void = wfc_img_load(c_filename.as_ptr());
            let wfc = wfc_overlapping(output_width,
                                      output_height,
                                      image,
                                      tile_width,
                                      tile_height,
                                      expand_input,
                                      xflip_tiles,
                                      yflip_tiles,
                                      rotate_tiles);

            return Wfc::new(wfc, image);
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
    let maybe_wfc = Wfc::overlapping(32, 32, "data/cave.png", 3, 3, 1, 1, 1, 1);
    assert!(maybe_wfc.is_some());
}

#[test]
pub fn test_run() {
    let maybe_wfc = Wfc::overlapping(32, 32, "data/cave.png", 3, 3, 1, 1, 1, 1);
    assert!(maybe_wfc.is_some());

    let mut wfc = maybe_wfc.unwrap();

    let result = wfc.run(Some(1));
    assert_eq!(Ok(()), result);
}

#[test]
pub fn test_export() {
    let maybe_wfc = Wfc::overlapping(32, 32, "data/cave.png", 3, 3, 1, 1, 1, 1);
    assert!(maybe_wfc.is_some());

    let mut wfc = maybe_wfc.unwrap();

    let result = wfc.run(Some(100));
    assert_eq!(Ok(()), result);

    wfc.export("output.png");
}

