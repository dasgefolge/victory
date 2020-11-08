use crate::state::{
    Input,
    InputRequest,
};

pub trait Frontend<P> {
    fn give_input(&mut self, ir: InputRequest) -> Input;
}
