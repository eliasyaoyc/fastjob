pub const SUCCESS: u64 = 200;
pub const FAIL: u64 = 401;

pub struct GrpcReturn {
    code: u64,
    msg: String,
}

impl GrpcReturn {
    pub fn empty() -> Option<GrpcReturn> {
        None
    }

    pub fn success() -> Option<GrpcReturn> {
        Some(GrpcReturn { code: SUCCESS, msg: "".to_string() })
    }

    pub fn success_with_params(code: u64, msg: String) -> Option<GrpcReturn> {
        Some(GrpcReturn { code, msg })
    }
}