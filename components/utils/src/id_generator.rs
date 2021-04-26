pub enum GeneratorTyp {
    Server,
    WorkerManager,
}

/// Generator unique id.
pub fn generator_id(typ: GeneratorTyp) -> usize {
    1
}