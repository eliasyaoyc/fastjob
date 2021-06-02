use grpcio::{RpcContext, UnarySink};
use futures::{TryFutureExt, FutureExt};
use crate::services::GRPC_RESPONSE_CODE;

#[derive(Clone)]
pub struct PingPongService {}

impl PingPong for PingPongService {
    fn ping_p(
        &mut self,
        ctx: RpcContext,
        req: Ping,
        sink: UnarySink<Pong>,
    ) {
        let msg = format!("Pong");

        let mut resp = Pong::default();
        resp.set_message(msg);
        resp.set_code(GRPC_RESPONSE_CODE);
        let f = sink
            .success(resp)
            .map_err(move |e| format!("failed to reply {:?}: {:?}", req, e))
            .map(|_| ());
        ctx.spawn(f)
    }
}