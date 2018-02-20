use algo::DH;

use communication::Communicate;
use communication::CommunicateEncr;

use failure::Error;

use bignum::NumBigUint as BigNum;

pub struct Server<T: Communicate> {
    stream: T,
    key: Vec<u8>,
}

impl<T: Communicate> Server<T> {
    pub fn new(mut stream: T) -> Result<Server<T>, Error> {
        handshake(&mut stream).map(|key| {
            Server {
                stream: stream,
                key: key,
            }
        })
    }
}

impl<T: Communicate> Communicate for Server<T> {
    fn send(&mut self, message: &[u8]) -> Result<(), Error> {
        self.stream.send_encr(message, &self.key)
    }

    fn receive(&mut self) -> Result<Option<Vec<u8>>, Error> {
        self.stream.receive_encr(&self.key)
    }
}

#[allow(non_snake_case)]
fn handshake<T: Communicate>(stream: &mut T) -> Result<Vec<u8>, Error> {
    let mut dh = DH::<BigNum>::new();
    let p = stream.receive()?.unwrap();
    let g = stream.receive()?.unwrap();
    dh.init_with_parameters(p, g);
    let A = dh.public_key();
    stream.send(&A)?;
    let B = stream.receive()?.unwrap();
    Ok(dh.shared_key(&B))
}
