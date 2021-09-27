mod spec;
mod berlin_spec;

pub use berlin_spec::{BerlinSpec,BerlinSpecStatic};
pub use spec::Spec;


//pub get_static

// call_static!(BerlinSpec, handler.call(test,test,test))
macro_rules! call_static {
    ($spec:ty, $object:ident.$func:ident($) ) => {
        
    };
}
