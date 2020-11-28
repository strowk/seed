use crate::{prelude::*, *};
use std::marker::PhantomData;
use js_sys::{Array, JsString, Reflect};
use std::rc::Rc;
use crate::app::DevTool;

pub struct WithDevtools {}

impl WithDevtools {
    pub fn new() -> WithDevtools {
        WithDevtools {}
    }
} 

impl<Ms, Mdl, INodes> crate::app::AppOptions<Ms, Mdl, INodes> for WithDevtools 
where
    Ms: serde::Serialize + 'static,
    Mdl: serde::Serialize + 'static,
    INodes: IntoNodes<Ms>,
{
    fn apply(&self, app: &mut App<Ms, Mdl, INodes>) {
        log!("Initializing Seed DevTools");
        let devtools = DevTools::new();
        devtools.initialized(app.data.model.borrow().as_ref().unwrap());
        app.devtools = Rc::new(Some(Box::new(devtools)));
    }
}

#[derive(Clone)]
pub struct DevTools<Ms, Mdl> {
    // DevTools has to be generic to allow creation for any Model and Msg,
    // but if it's generic, Rust requires to use all parameter types, hence
    // following phonies are included 
    phantom_ms: PhantomData<Ms>,
    phantom_mdl: PhantomData<Mdl>,
    model_key: JsString,
    messages_key: JsString,
    root_key: JsString
}

#[derive(serde::Serialize)]
struct InitialData<Ms> {
    seed_version: String,
    messages: Vec<Ms>,
}


#[derive(serde::Serialize)]
struct DevToolsAction {
    r#type: String,
    action: String,
}

impl<Ms, Mdl> DevTools<Ms, Mdl> {
    pub fn new() -> DevTools<Ms, Mdl> {
        DevTools{
            phantom_mdl: PhantomData,
            phantom_ms: PhantomData,
            root_key: JsString::from("SEED_DEVTOOLS_EXTENSION"),
            model_key: JsString::from("model"),
            messages_key: JsString::from("messages"),
        }
    }
}

fn extract_name(key: &JsString) -> String {
    match key.as_string() {
        Some(name) => name,
        None => "?".into()
    }
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

impl<Ms: serde::Serialize, Mdl: serde::Serialize> DevTools<Ms, Mdl> {
    fn get_root(&self) -> Result<JsValue, String> {
        let w = window();
        match Reflect::get(&w, &self.root_key) {
            Ok(value) if value.is_object() => {
                Ok(value)
            },
            _ => Err(format!("Window object doesn't have a suitable \"{}\" property", extract_name(&self.root_key)))
        }
    }

    fn init(&self) -> Result<(),()> {
        let w = window();
        let data: InitialData<Ms> = InitialData{
            seed_version: VERSION.to_string(), // TODO: somehow get it from the build
            messages: vec![],
        };
        let starting_val = serde_wasm_bindgen::to_value(&data);

        match starting_val {
            Err(js_val) => {
                Err(error!("Seed Devtools failed to serialize model", js_val))
            },
            Ok(starting_val) => match Reflect::set(&w, &self.root_key, &starting_val) {
                Ok(_) => Ok(()),
                Err(js_val) => Err(error!("Seed Devtools cannot set property ", extract_name(&self.root_key), " to Window", js_val))
            }
        }

        
    }
    
    fn save_model(&self, model: &Mdl) {
        let val = serde_wasm_bindgen::to_value(model);

        let root = self.get_root();

        match root {
            Err(s) => error!("Seed Devtools failed to retrieve the root object", s),
            Ok(root) => {
                match val {
                    Err(js_val) => {
                        error!("Seed Devtools failed to serialize model", js_val)
                    }
                    Ok(model) => {
                        match Reflect::set(&root, &self.model_key, &model) {
                            Ok(_) => (),
                            Err(js_val) => error!("Seed Devtools cannot set property ", extract_name(&self.model_key), " to Window", js_val)
                        };
                    }
                }
            }
        }

        let update = DevToolsAction{
            r#type: "TO_SEED_DEVTOOLS".to_string(),
            action: "RELOAD_MODEL".to_string(),
        };
        let update = serde_wasm_bindgen::to_value(&update);


        let w = window();
        let _ = update.and_then(|update|{
            let _ = w.post_message(&update, "*").or_else(|e|{
                Err(error!("Could not send dev tools RELOAD_MODEL action message", e))
            });
            Ok(())
        }).or_else(|e|{
            Err(error!("Could not serialize dev tools RELOAD_MODEL action message", e))
        });
    }

    fn save_message(&self, message: &Ms, _model_before: &Mdl) {
        // TODO: should save model before and after too

        let root = self.get_root();

        match root {
            Err(s) => error!("Seed Devtools failed to retrieve extension root object", s),
            Ok(root) => { 
                match Reflect::get(&root, &self.messages_key) {
                    Ok(value) => {
                        let messages: Array = Array::from(&value);
                        let val = serde_wasm_bindgen::to_value(message);
                        let _ = val.and_then(|val| {messages.push(&val); Ok(())}).or_else(|e| {
                            Err(error!("Seed Devtools could not send message to extension", e))
                        });

                        let _ = Reflect::set(&root, &self.messages_key, &messages)
                            .or_else(|e|{
                                let log_msg = format!("Seed Devtools failed to set field {} on root extension object", extract_name(&self.messages_key));
                                Err(error!(log_msg, e))
                            }).is_ok();

                    },
                    _ => error!(format!("Seed Devtools could not find \"{}\" property in extension root object", extract_name(&self.messages_key)))
                }
            }        
        }
    }
}


impl<Ms: serde::Serialize, Mdl: serde::Serialize> DevTool for DevTools<Ms, Mdl> {
    type Mdl = Mdl;
    type Ms =  Ms;
    
    fn initialized(&self, model: &Mdl) {
        if self.init().is_err() {
            return;
        }
        self.save_model(model);
    }

    fn received(&self, ms: &Ms, model: &Mdl) {
        self.save_message(ms, model);
    }

    fn updated(&self, model: &Mdl) {
        self.save_model(model);
    }

}
