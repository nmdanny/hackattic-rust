use std::fmt;
extern crate hex;

pub struct HexSlice<'a>(pub &'a [u8]);

impl<'a> HexSlice<'a> {
    pub fn new<T>(data: &'a T) -> HexSlice<'a> 
        where T: ?Sized + AsRef<[u8]> + 'a
    {
        HexSlice(data.as_ref())
    }
}

// You can even choose to implement multiple traits, like Lower and UpperHex
impl<'a> fmt::Display for HexSlice<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.0 {
            // Decide if you want to pad out the value here
            write!(f, "{:x}", byte)?;
        }
        Ok(())
    }
}
