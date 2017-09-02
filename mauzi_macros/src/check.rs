use Result;
use ast;

pub fn check(ast: &ast::Dict) -> Result<()> {
    custom_return_implies_raw_body(ast)?;

    Ok(())
}

/// Translation unit arms can have string or raw bodies. The latter is raw
/// Rust code. Since string bodies always produce a `String` it doesn't make
/// sense to use those in combination with custom return types.
///
/// We make sure those are never used in combination by checking it here.
fn custom_return_implies_raw_body(ast: &ast::Dict) -> Result<()> {
    for unit in ast.units().filter(|unit| unit.return_type.is_some()) {
        let not_raw = unit.body.arms.iter()
            .find(|arm| !arm.body.is_raw_block());

        if let Some(not_raw) = not_raw {
            return Err(format!(
                "translation unit '{}' has custom return type, but its arm \
                    '{}' doesn't have a raw body (required)",
                unit.name,
                not_raw.pattern,
            ))
        }
    }

    Ok(())
}
