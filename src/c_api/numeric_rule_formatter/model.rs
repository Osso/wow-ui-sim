use rilua::{LuaResult, runtime_error};

#[derive(Clone, Copy, Debug)]
pub(super) enum Rounding {
    Nearest,
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub(super) struct Component {
    pub div: Option<f64>,
    pub modulo: Option<f64>,
    pub step: Option<f64>,
    pub rounding: Rounding,
}

#[derive(Clone, Debug)]
pub(super) struct Breakpoint {
    pub threshold: f64,
    pub step: Option<f64>,
    pub rounding: Rounding,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub format: String,
    pub components: Option<Vec<Component>>,
}

impl Breakpoint {
    pub fn arguments(&self, input: f64) -> LuaResult<Vec<f64>> {
        let mut value = round_to_step(input, self.step, self.rounding)?;
        if let Some(min) = self.min {
            value = value.max(min);
        }
        if let Some(max) = self.max {
            value = value.min(max);
        }
        match &self.components {
            Some(components) => components.iter().map(|part| part.apply(value)).collect(),
            None => Ok(vec![value]),
        }
    }
}

impl Component {
    fn apply(&self, input: f64) -> LuaResult<f64> {
        let quotient = self.div.map_or(input, |div| input / div);
        let remainder = self.modulo.map_or(quotient, |modulo| quotient % modulo);
        round_to_step(remainder, self.step, self.rounding)
    }
}

fn round_to_step(value: f64, step: Option<f64>, mode: Rounding) -> LuaResult<f64> {
    let Some(step) = step else {
        return finite(value);
    };
    let scaled = finite(value / step)?;
    let rounded = match mode {
        Rounding::Up => scaled.ceil(),
        Rounding::Down => scaled.floor(),
        Rounding::Nearest => {
            if scaled.fract().abs() == 0.5 {
                return Err(runtime_error(
                    "NumericRuleFormatter nearest half-step rounding is unverified",
                ));
            }
            scaled.round()
        }
    };
    finite(rounded * step)
}

pub(super) fn finite(value: f64) -> LuaResult<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(runtime_error(
            "NumericRuleFormatter requires finite numeric values",
        ))
    }
}

pub(super) fn validate_format(format: &str, component_count: Option<usize>) -> LuaResult<()> {
    let bytes = format.as_bytes();
    let mut position = 0;
    let mut conversions = 0;
    while position < bytes.len() {
        if bytes[position] != b'%' {
            position += 1;
            continue;
        }
        position += 1;
        if bytes.get(position) == Some(&b'%') {
            position += 1;
            continue;
        }
        position = numeric_conversion_end(bytes, position)?;
        conversions += 1;
    }
    let valid_count = component_count.map_or(conversions <= 1, |count| conversions == count);
    if !valid_count {
        return Err(runtime_error(
            "NumericRuleFormatter format/component count mismatch",
        ));
    }
    Ok(())
}

fn numeric_conversion_end(bytes: &[u8], mut position: usize) -> LuaResult<usize> {
    while bytes
        .get(position)
        .is_some_and(|byte| b"-+ #0.123456789".contains(byte))
    {
        position += 1;
    }
    if bytes
        .get(position)
        .is_some_and(|byte| b"diouxXeEfgG".contains(byte))
    {
        Ok(position + 1)
    } else {
        Err(runtime_error(
            "NumericRuleFormatter format must use numeric conversion specifiers",
        ))
    }
}
