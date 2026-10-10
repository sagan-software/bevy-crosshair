//! Reflection preserves validated settings while allowing whole-value replacement.

use super::*;

/// Covers opaque configuration, host input, marker appearance, and typed hit reflection.
pub(crate) fn run() {
    // Opaque configuration cannot expose independently mutable private setting fields.
    let original = Crosshair::default();
    assert!(matches!(
        original.reflect_ref(),
        bevy::reflect::ReflectRef::Opaque(_)
    ));
    replace(original, Crosshair::default().with_style(Style::Dot));
    replace(CrosshairInput::default(), CrosshairInput::default());
    // Marker and event reflection retain metadata without requiring reflection for delivery.
    replace(HitMarker::default(), HitMarker::default().scaled(2.0));
    let entity = Entity::PLACEHOLDER;
    replace(
        Hit {
            entity,
            metadata: HitMarker::default(),
        },
        Hit {
            entity,
            metadata: HitMarker::default(),
        },
    );
}

/// Checks successful reconstruction and rejection of an incompatible reflected type.
fn replace<T: Reflect + FromReflect>(mut original: T, next: T) {
    // Reconstruction and dynamic conversion must keep the concrete type recoverable.
    let restored = T::from_reflect(&original).expect("the same reflected type reconstructs");
    original
        .set(Box::new(next))
        .expect("the replacement has the same concrete type");
    original
        .set(Box::new(0_u32))
        .expect_err("invalid input must be rejected");
    let dynamic = original
        .to_dynamic()
        .expect("the concrete value supports reconstruction");
    assert!(T::from_reflect(dynamic.as_ref()).is_some());
    // Failed application must remain a typed reflection error, never a partial mutation.
    original
        .try_apply(&restored)
        .expect("the same type applies");
    original
        .try_apply(&0_u32)
        .expect_err("invalid input must be rejected");
}
