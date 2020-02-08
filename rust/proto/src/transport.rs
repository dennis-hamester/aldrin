use super::Message;
use futures_core::stream::Stream;
use futures_sink::Sink;

pub trait Transport:
    Stream<Item = Result<Message, <Self as Sink<Message>>::Error>> + Sink<Message>
{
    fn name(&self) -> Option<&str> {
        None
    }
}
