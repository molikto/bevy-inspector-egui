//! Custom UI implementations for specific types. Check [`InspectorPrimitive`] for an example.

use crate::{
    reflect_inspector::{InspectorUi, errors::no_multiedit},
    utils::pretty_type_name,
};
use bevy_platform::time::Instant;
use bevy_reflect::{FromType, PartialReflect, Reflect, TypePath, TypeRegistry};
use std::{
    any::{Any, TypeId},
    borrow::Cow,
    path::PathBuf,
};

mod std_impls;

type InspectorEguiImplFn =
    fn(&mut dyn Any, &mut egui::Ui, &dyn Any, egui::Id, InspectorUi<'_, '_>) -> bool;
type InspectorEguiImplFnReadonly =
    fn(&dyn Any, &mut egui::Ui, &dyn Any, egui::Id, InspectorUi<'_, '_>);


/// Custom UI implementation for a concrete type.
///
/// # Example Usage
/// ```rust,no_run
/// use bevy::prelude::*;
/// use bevy_inspector_egui::inspector_egui_impls::{InspectorEguiImpl, InspectorPrimitive};
/// use bevy_inspector_egui::quick::ResourceInspectorPlugin;
/// use bevy_inspector_egui::reflect_inspector::InspectorUi;
///
/// #[derive(Reflect, Default)]
/// struct ToggleOption(bool);
///
/// impl InspectorPrimitive for ToggleOption {
///     fn ui(&mut self, ui: &mut egui::Ui, _: &dyn std::any::Any, _: egui::Id, _: InspectorUi<'_, '_>) -> bool {
///         let mut changed = ui.radio_value(&mut self.0, false, "Disabled").changed();
///         changed |= ui.radio_value(&mut self.0, true, "Enabled").changed();
///         changed
///     }
///
///     fn ui_readonly(&self, ui: &mut egui::Ui, _: &dyn std::any::Any, _: egui::Id, _: InspectorUi<'_, '_>) {
///         let mut copy = self.0;
///         ui.add_enabled_ui(false, |ui| {
///             ui.radio_value(&mut copy, false, "Disabled").changed();
///             ui.radio_value(&mut copy, true, "Enabled").changed();
///         });
///     }
/// }
///
/// fn main() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         // ...
///         .register_type_data::<ToggleOption, InspectorEguiImpl>()
///         .run();
/// }
/// ```
pub trait InspectorPrimitive: Reflect {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool;
    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    );
}

fn ui_vtable<T: InspectorPrimitive>(
    val: &mut dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
) -> bool {
    let val = val.downcast_mut::<T>().unwrap();
    T::ui(val, ui, options, id, env)
}
fn ui_readonly_vtable<T: InspectorPrimitive>(
    val: &dyn Any,
    ui: &mut egui::Ui,
    options: &dyn Any,
    id: egui::Id,
    env: InspectorUi<'_, '_>,
) {
    let val = val.downcast_ref::<T>().unwrap();
    T::ui_readonly(val, ui, options, id, env)
}

/// Function pointers for displaying a concrete type, to be registered in the [`TypeRegistry`].
///
/// This can used for leaf types like `u8` or `String`, as well as people who want to completely customize the way
/// to display a certain type. You can use [`InspectorPrimitive`] to avoid manually writing the function pointers with correct downcasting.
#[derive(Clone)]
pub struct InspectorEguiImpl {
    fn_mut: InspectorEguiImplFn,
    fn_readonly: InspectorEguiImplFnReadonly,
}

impl<T: InspectorPrimitive> FromType<T> for InspectorEguiImpl {
    fn from_type() -> Self {
        InspectorEguiImpl::of::<T>()
    }
}

impl InspectorEguiImpl {
    pub fn of<T: InspectorPrimitive>() -> Self {
        InspectorEguiImpl {
            fn_mut: ui_vtable::<T>,
            fn_readonly: ui_readonly_vtable::<T>,
        }
    }

    /// Create a new [`InspectorEguiImpl`] from functions displaying a type
    pub fn new(
        fn_mut: InspectorEguiImplFn,
        fn_readonly: InspectorEguiImplFnReadonly,
    ) -> Self {
        InspectorEguiImpl {
            fn_mut,
            fn_readonly,
        }
    }

    pub fn execute<'a, 'c: 'a>(
        &'a self,
        value: &mut dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) -> bool {
        (self.fn_mut)(value, ui, options, id, env)
    }
    pub fn execute_readonly<'a, 'c: 'a>(
        &'a self,
        value: &dyn Any,
        ui: &mut egui::Ui,
        options: &dyn Any,
        id: egui::Id,
        env: InspectorUi<'_, '_>,
    ) {
        (self.fn_readonly)(value, ui, options, id, env)
    }

}

fn add<T: InspectorPrimitive + TypePath + PartialEq + Clone>(type_registry: &mut TypeRegistry) {
    type_registry.register_type_data::<T, InspectorEguiImpl>();
}

/// Register [`InspectorEguiImpl`]s for primitive rust types as well as standard library types
#[rustfmt::skip]
pub fn register_std_impls(type_registry: &mut TypeRegistry) {
    add::<f32>(type_registry);
    add::<f64>(type_registry);
    add::<i8>(type_registry);
    add::<i16>(type_registry);
    add::<i32>(type_registry);
    add::<i64>(type_registry);
    add::<isize>(type_registry);
    add::<u8>(type_registry);
    add::<u16>(type_registry);
    add::<u32>(type_registry);
    add::<u64>(type_registry);
    add::<usize>(type_registry);
    add::<bool>(type_registry);
    add::<String>(type_registry);
    add::<Cow<str>>(type_registry);
    type_registry.register::<PathBuf>();
    add::<PathBuf>(type_registry);

    type_registry.register::<std::ops::Range<f64>>();
    type_registry.register::<std::ops::RangeInclusive<f32>>();
    type_registry.register::<std::ops::RangeInclusive<f64>>();
    add::<std::ops::Range<f32>>(type_registry);
    add::<std::ops::Range<f64>>(type_registry);
    add::<std::ops::RangeInclusive<f32>>(type_registry);
    add::<std::ops::RangeInclusive<f64>>(type_registry);
    add::<TypeId>(type_registry);

    add::<std::time::Duration>(type_registry);
    add::<Instant>(type_registry);
}


pub(crate) fn change_slider<T>(
    ui: &mut egui::Ui,
    id: egui::Id,
    same: Option<T>,
    f: impl FnOnce(T, bool),
) -> bool
where
    T: egui::emath::Numeric + std::ops::Sub<Output = T> + Default + Send + Sync + 'static,
{
    let speed = if T::INTEGRAL { 1.0 } else { 0.1 };

    match same {
        Some(mut same) => {
            let widget = egui::DragValue::new(&mut same).speed(speed);

            let changed = ui.add(widget).changed();
            if changed {
                f(same, true);
            }

            changed
        }
        None => {
            let old_change = ui.memory_mut(|memory| *memory.data.get_temp_mut_or_default::<T>(id));
            let mut change = old_change;

            let widget = egui::DragValue::new(&mut change)
                .speed(speed)
                .custom_formatter(|_, _| "-".to_string());

            let changed = ui.add(widget).changed();
            if changed {
                f(change - old_change, false);
            }

            ui.memory_mut(|memory| *memory.data.get_temp_mut_or_default(id) = change);
            changed
        }
    }
}

pub(crate) fn iter_all_eq<T: PartialEq>(mut iter: impl Iterator<Item = T>) -> Option<T> {
    let first = iter.next()?;
    iter.all(|elem| elem == first).then_some(first)
}

#[macro_export]
#[doc(hidden)]
macro_rules! many_ui {
    ($name:ident $inner:ident $ty:ty) => {
        pub fn $name(
            ui: &mut egui::Ui,
            options: &dyn Any,
            id: egui::Id,
            env: InspectorUi<'_, '_>,
            values: &mut [&mut dyn bevy_reflect::PartialReflect],
            projector: &dyn $crate::reflect_inspector::ProjectorReflect,
        ) -> bool {
            let same = $crate::inspector_egui_impls::iter_all_eq(
                values
                    .iter_mut()
                    .map(|value| projector(*value).try_downcast_ref::<$ty>().unwrap()),
            );

            let mut temp = same.cloned().unwrap_or_default();
            if $inner(&mut temp, ui, options, id, env) {
                for value in values.iter_mut() {
                    let value = projector(*value).try_downcast_mut::<$ty>().unwrap();
                    *value = temp.clone();
                }

                return true;
            }
            false
        }
    };
}
