use std::marker::PhantomData;

use crate::bindings::{
    CgosI2CCount, CgosI2CGetFrequency, CgosI2CGetMaxFrequency, CgosI2CIsAvailable, CgosI2CRead,
    CgosI2CReadRegister, CgosI2CSetFrequency, CgosI2CType, CgosI2CWrite, CgosI2CWriteReadCombined,
    CgosI2CWriteRegister, CGOS_I2C_TYPE_DDC, CGOS_I2C_TYPE_PRIMARY, CGOS_I2C_TYPE_SMB,
    CGOS_I2C_TYPE_UNKNOWN,
};

/// Error type for I2c operations
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Error {
    /// User-supplied index is out of range
    IndexOutOfRange,
    /// I2c bus transaction failed
    Bus,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum I2cKind {
    Unknown,
    Primary,
    Smb,
    Ddc,
    CongatecInternalUse(u32),
}

impl From<I2cKind> for u32 {
    fn from(val: I2cKind) -> Self {
        match val {
            I2cKind::Unknown => CGOS_I2C_TYPE_UNKNOWN,
            I2cKind::Primary => CGOS_I2C_TYPE_PRIMARY,
            I2cKind::Smb => CGOS_I2C_TYPE_SMB,
            I2cKind::Ddc => CGOS_I2C_TYPE_DDC,
            I2cKind::CongatecInternalUse(x) => x,
        }
    }
}

impl From<u32> for I2cKind {
    // note: On my devboard libcgos does return undeclared values for some busses, hence the need to return an error instead of panic
    fn from(value: u32) -> I2cKind {
        match value {
            CGOS_I2C_TYPE_UNKNOWN => Self::Unknown,
            CGOS_I2C_TYPE_PRIMARY => Self::Primary,
            CGOS_I2C_TYPE_SMB => Self::Smb,
            CGOS_I2C_TYPE_DDC => Self::Ddc,
            _ => Self::CongatecInternalUse(value),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct I2c<'library> {
    handle: u32,
    index: u32,
    _library_lifetime: PhantomData<&'library ()>,
}

impl I2c<'_> {
    pub(crate) fn new(handle: u32, index: usize) -> Result<Self> {
        let num_busses = Self::amount(handle);
        if index > num_busses.saturating_sub(1) {
            return Err(Error::IndexOutOfRange);
        }

        let index = index.try_into().map_err(|_| Error::IndexOutOfRange)?;
        let ret = Self {
            handle,
            index,
            _library_lifetime: PhantomData,
        };
        Ok(ret)
    }

    pub(crate) fn amount(handle: u32) -> usize {
        unsafe { CgosI2CCount(handle) as usize }
    }

    pub fn i2c_type(&self) -> I2cKind {
        let raw = unsafe { CgosI2CType(self.handle, self.index) };
        I2cKind::from(raw)
    }

    pub fn is_available(&self) -> bool {
        let raw = unsafe { CgosI2CIsAvailable(self.handle, self.index) };
        raw == 1
    }

    pub fn read(&self, bus_address: u8, data: &mut [u8]) -> Result<()> {
        let retcode = unsafe {
            CgosI2CRead(
                self.handle,
                self.index,
                bus_address,
                data.as_mut_ptr(),
                data.len() as u32,
            )
        };

        if retcode == 0 {
            return Err(Error::Bus);
        }

        Ok(())
    }

    pub fn write(&self, bus_addr: u8, wr_data: &[u8]) -> Result<()> {
        let retcode = unsafe {
            CgosI2CWrite(
                self.handle,
                self.index,
                bus_addr,
                wr_data.as_ptr() as *mut u8,
                wr_data.len() as u32,
            )
        };

        if retcode == 0 {
            return Err(Error::Bus);
        }

        Ok(())
    }

    pub fn read_register(&self, bus_addr: u8, reg_addr: u16) -> Result<u8> {
        let mut data: u8 = 0;
        let retcode = unsafe {
            CgosI2CReadRegister(
                self.handle,
                self.index,
                bus_addr,
                reg_addr,
                &mut data as *mut u8,
            )
        };

        if retcode == 0 {
            return Err(Error::Bus);
        }

        Ok(data)
    }

    pub fn write_register(&self, bus_addr: u8, reg_addr: u16, val: u8) -> Result<()> {
        let retcode =
            unsafe { CgosI2CWriteRegister(self.handle, self.index, bus_addr, reg_addr, val) };

        if retcode == 0 {
            return Err(Error::Bus);
        }

        Ok(())
    }

    pub fn write_read_combined(
        &self,
        bus_addr: u8,
        wr_data: &[u8],
        rd_data: &mut [u8],
    ) -> Result<()> {
        let wr_len = wr_data.len();
        let retcode = unsafe {
            CgosI2CWriteReadCombined(
                self.handle,
                self.index,
                bus_addr,
                wr_data.as_ptr() as *mut u8,
                wr_len.try_into().unwrap(),
                rd_data.as_mut_ptr(),
                rd_data.len() as u32,
            )
        };

        if retcode == 0 {
            return Err(Error::Bus);
        }

        Ok(())
    }

    pub fn get_max_frequency(&self) -> Result<u32> {
        let mut ret = 0;
        let retcode =
            unsafe { CgosI2CGetMaxFrequency(self.handle, self.index, &mut ret as *mut u32) };

        if retcode == 0 {
            return Err(Error::Bus);
        }

        Ok(ret)
    }

    pub fn get_frequency(&self) -> Result<u32> {
        let mut ret = 0;
        let retcode = unsafe { CgosI2CGetFrequency(self.handle, self.index, &mut ret as *mut u32) };

        if retcode == 0 {
            return Err(Error::Bus);
        }

        Ok(ret)
    }

    pub fn set_frequency(&self, frequency: u32) -> Result<()> {
        let retcode = unsafe { CgosI2CSetFrequency(self.handle, self.index, frequency) };

        if retcode == 0 {
            return Err(Error::Bus);
        }

        Ok(())
    }
}
