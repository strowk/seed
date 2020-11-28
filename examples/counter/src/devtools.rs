use seed::{prelude::*, *};
use std::marker::PhantomData;

pub struct DevTools<Ms, Mdl> {
    // DevTools has to be generic to allow creation for any Model and Msg,
    // but if it's generic, Rust requires to use all parameter types, hence
    // following phonies are included 
    phantom_ms: PhantomData<Ms>,
    phantom_mdl: PhantomData<Mdl>
}

impl<Ms, Mdl> DevTools<Ms, Mdl> {
    pub fn new() -> DevTools<Ms, Mdl> {
        DevTools{
            phantom_mdl: PhantomData,
            phantom_ms: PhantomData,
        }
    }
}

// #[wasm_bindgen]

impl<Ms: serde::Serialize, Mdl: serde::Serialize> seed::Middleware for DevTools<Ms, Mdl> {
    type Mdl = Mdl;
    type Ms =  Ms;
    
    fn initialized(&self, model: &Mdl) {
        // TODO: here we would send resulted js to extension somehow
        log!(serde_wasm_bindgen::to_value(model));
    }

    fn received(&self, ms: &Self::Ms) {
        // TODO: here we would send resulted js to extension somehow
        log!(serde_wasm_bindgen::to_value(ms));
    }

}