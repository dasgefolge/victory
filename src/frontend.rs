use crate::state::{
    Input,
    InputRequest,
    PlayerId,
};

pub trait Frontend<P: PlayerId> {
    fn give_input(&mut self, ir: InputRequest) -> Input<P>;
}
