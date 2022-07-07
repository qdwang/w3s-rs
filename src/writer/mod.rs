use std::io;

pub mod car;

pub mod general;

#[cfg(feature = "encryption")]
pub mod crypto;

#[cfg(feature = "compression")]
pub mod compression;

pub trait ChainWrite<W: io::Write>: io::Write {
    fn pass2next_writer(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(next_writer) = self.next_writer() {
            next_writer.write(buf)?;
            Ok(buf.len())
        } else {
            Ok(0)
        }
    }
    fn next_writer(&mut self) -> Option<&mut W>;
    fn next(self) -> Option<W>;
}

macro_rules! take_nth_next {
    ($w:ident $($tails:tt)*) => {
        take_nth_next!(@next(ChainWrite::next($w)) $($tails)*)
    };
    (@next($($x:tt)*) > $($tails:tt)*) => {
        take_nth_next!(@next(Option::and_then($($x)*, |x| x.next())) $($tails)*)
    };
    (@next($($x:tt)*)) => {
        $($x)*
    };
}
pub(crate) use take_nth_next; 

