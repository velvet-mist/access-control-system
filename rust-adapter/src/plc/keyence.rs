use crate::error::AdapterError;

pub struct KeyencePlc{
    access_req: bool,
    access_ok:bool,
    adapter_ok:bool,
}

impl KeyencePlc {
    pub fn new() -> Self {
        Self{
            access_ok:false,
            access_req:false,
            adapter_ok:false,
        }
    }

    pub fn set_allow(&self) -> Result<(), AdapterError> {
        println!("PLC: ACCESS ALLOWED");
        Ok(())
    }

    pub fn set_deny(&self) -> Result<(), AdapterError> {
        println!("PLC: ACCESS DENIED");
        Ok(())
    }
}
