use snowflake::{SnowflakeIdGenerator, SnowflakeIdBucket};

pub enum GeneratorTyp {
    Server,
    WorkerManager,
}

pub struct IdGenerator {
    server_generator: SnowflakeIdBucket,
    task_generator: SnowflakeIdBucket,
}

/// Generator unique id use snowflake.
pub fn generator_id(typ: GeneratorTyp) -> u64 {
    let mut id_generator_generator = SnowflakeIdGenerator::new(1, 1);
    let id = id_generator_generator.real_time_generate();

    let mut id_generator_bucket = SnowflakeIdBucket::new(1, 1);
    let id = id_generator_bucket.get_id();
    1
}
