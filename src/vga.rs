use std::mem::zeroed;

use crate::bindings::{
    CgosVgaCount, CgosVgaGetBacklightEnable, CgosVgaGetContrast, CgosVgaGetContrastEnable,
    CgosVgaGetInfo, CgosVgaSetBacklight, CgosVgaSetBacklightEnable, CgosVgaSetContrast,
    CgosVgaSetContrastEnable, CGOSVGAINFO, CGOS_VGA_TYPE_CRT, CGOS_VGA_TYPE_LCD,
    CGOS_VGA_TYPE_LCD_DVO, CGOS_VGA_TYPE_LCD_LVDS, CGOS_VGA_TYPE_TV, CGOS_VGA_TYPE_UNKNOWN,
};

/// Error type for I2c operations
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum VgaErr {
    /// User-supplied index is out of range
    IdxOutOfRange,

    /// Any error returned by underlying libcgos calls
    LibcgosErr,
}

pub type VgaResult<T> = Result<T, VgaErr>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Vga {
    handle: u32,
    index: u32,
}
impl Vga {
    pub(crate) fn amount(handle: u32) -> usize {
        unsafe { CgosVgaCount(handle) as usize }
    }

    pub(crate) fn new(handle: u32, index: usize) -> VgaResult<Self> {
        let num_vga = Self::amount(handle);
        if index > num_vga.saturating_sub(1) {
            return Err(VgaErr::IdxOutOfRange);
        }

        let index = index.try_into().map_err(|_| VgaErr::IdxOutOfRange)?;
        let ret = Self {
            handle,
            index: index,
        };
        Ok(ret)
    }

    pub fn get_contrast(&self) -> VgaResult<u32> {
        let ret = 0u32;
        let retcode = unsafe { CgosVgaGetContrast(self.handle, self.index, ret as *mut u32) };

        if retcode != 0 {
            return Ok(ret);
        } else {
            return Err(VgaErr::LibcgosErr);
        }
    }

    pub fn set_contrast(&self, value: u32) -> VgaResult<()> {
        let retcode = unsafe { CgosVgaSetContrast(self.handle, self.index, value) };

        if retcode != 0 {
            return Ok(());
        } else {
            return Err(VgaErr::LibcgosErr);
        }
    }

    pub fn get_contrast_enable(&self) -> VgaResult<bool> {
        let ret = 0;
        let retcode = unsafe { CgosVgaGetContrastEnable(self.handle, self.index, ret as *mut u32) };

        if retcode != 0 {
            return Ok(ret != 0);
        } else {
            return Err(VgaErr::LibcgosErr);
        }
    }

    pub fn set_contrast_enable(&self, en: bool) -> VgaResult<()> {
        let retcode = unsafe { CgosVgaSetContrastEnable(self.handle, self.index, u32::from(en)) };

        if retcode != 0 {
            return Ok(());
        } else {
            return Err(VgaErr::LibcgosErr);
        }
    }

    pub fn set_backlight(&self, value: u32) -> VgaResult<()> {
        let retcode = unsafe { CgosVgaSetBacklight(self.handle, self.index, value) };

        if retcode != 0 {
            return Ok(());
        } else {
            return Err(VgaErr::LibcgosErr);
        }
    }

    pub fn get_backlight_enable(&self) -> VgaResult<bool> {
        let ret = 0;
        let retcode =
            unsafe { CgosVgaGetBacklightEnable(self.handle, self.index, ret as *mut u32) };

        if retcode != 0 {
            return Ok(ret != 0);
        } else {
            return Err(VgaErr::LibcgosErr);
        }
    }

    pub fn set_backlight_enable(&self, en: bool) -> VgaResult<()> {
        let retcode = unsafe { CgosVgaSetBacklightEnable(self.handle, self.index, u32::from(en)) };

        if retcode != 0 {
            return Ok(());
        } else {
            return Err(VgaErr::LibcgosErr);
        }
    }

    pub fn get_info(&self) -> VgaResult<VgaInfo> {
        let mut info: CGOSVGAINFO = unsafe { zeroed() };
        info.dwSize = size_of::<CGOSVGAINFO>() as u32;

        let retcode = unsafe { CgosVgaGetInfo(self.handle, self.index, &mut info) };
        if retcode == 0 {
            return Ok(info.into());
        } else {
            return Err(VgaErr::LibcgosErr);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum VgaType {
    Unknown,
    Crt,
    Lcd,
    LcdDv0,
    LcdLvds,
    Tv,
    Other(u32),
}

impl Into<u32> for VgaType {
    fn into(self) -> u32 {
        match self {
            Self::Unknown => CGOS_VGA_TYPE_UNKNOWN,
            Self::Crt => CGOS_VGA_TYPE_CRT,
            Self::Lcd => CGOS_VGA_TYPE_LCD,
            Self::LcdDv0 => CGOS_VGA_TYPE_LCD_DVO,
            Self::LcdLvds => CGOS_VGA_TYPE_LCD_LVDS,
            Self::Tv => CGOS_VGA_TYPE_TV,
            Self::Other(x) => x,
        }
    }
}

impl From<u32> for VgaType {
    //note: On my devboard libcgos does return undeclared values for some busses, hence the need to return an error instead of panic
    fn from(value: u32) -> VgaType {
        match value {
            CGOS_VGA_TYPE_UNKNOWN => Self::Unknown,
            CGOS_VGA_TYPE_CRT => Self::Crt,
            CGOS_VGA_TYPE_LCD => Self::Lcd,
            CGOS_VGA_TYPE_LCD_DVO => Self::LcdDv0,
            CGOS_VGA_TYPE_LCD_LVDS => Self::LcdLvds,
            CGOS_VGA_TYPE_TV => Self::Tv,
            _ => Self::Other(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VgaInfo {
    size: u32,
    vga_type: u32,
    flags: u32,
    native_width: u32,
    native_height: u32,
    requested_width: u32,
    requested_height: u32,
    requested_bpp: u32,
    max_backlight: u32,
    max_contrast: u32,
}

impl From<CGOSVGAINFO> for VgaInfo {
    fn from(info: CGOSVGAINFO) -> VgaInfo {
        VgaInfo {
            size: info.dwSize,
            vga_type: info.dwType,
            flags: info.dwFlags,
            native_width: info.dwNativeWidth,
            native_height: info.dwNativeHeight,
            requested_width: info.dwRequestedWidth,
            requested_height: info.dwRequestedHeight,
            requested_bpp: info.dwRequestedBpp,
            max_backlight: info.dwMaxBacklight,
            max_contrast: info.dwMaxContrast,
        }
    }
}
