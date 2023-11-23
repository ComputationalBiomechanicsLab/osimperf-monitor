#[derive(Copy, Clone, Debug, Default)]
pub struct TimeAndValue<T = f64> {
    pub time: f64,
    pub value: T,
}
