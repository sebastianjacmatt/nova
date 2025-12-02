use crate::{
    ecmascript::{
        builders::builtin_function_builder::BuiltinFunctionBuilder,
        builtins::{ArgumentsList, Behaviour, Builtin, BuiltinIntrinsicConstructor},
        execution::{Agent, JsResult, Realm},
        types::{BUILTIN_STRING_MEMORY, IntoObject, Object, String, Value},
    },
    engine::context::{GcScope, NoGcScope},
    heap::IntrinsicConstructorIndexes,
};

pub(crate) struct TemporalZonedDateTimeConstructor;

impl Builtin for TemporalZonedDateTimeConstructor {
    const NAME: String<'static> = BUILTIN_STRING_MEMORY.ZonedDateTime;
    const LENGTH: u8 = 1;
    const BEHAVIOUR: Behaviour =
        Behaviour::Constructor(TemporalZonedDateTimeConstructor::constructor);
}

impl BuiltinIntrinsicConstructor for TemporalZonedDateTimeConstructor {
    const INDEX: IntrinsicConstructorIndexes = IntrinsicConstructorIndexes::TemporalZonedDateTime;
}

impl TemporalZonedDateTimeConstructor {
    fn constructor<'gc>(
        _agent: &mut Agent,
        _: Value,
        _args: ArgumentsList,
        _new_target: Option<Object>,
        mut _gc: GcScope<'gc, '_>,
    ) -> JsResult<'gc, Value<'gc>> {
        unimplemented!()
    }

    pub(crate) fn create_intrinsic(agent: &mut Agent, realm: Realm<'static>, _gc: NoGcScope) {
        let intrinsics = agent.get_realm_record_by_id(realm).intrinsics();
        let zoned_date_time_prototype = intrinsics.temporal_zoned_date_time_prototype();

        BuiltinFunctionBuilder::new_intrinsic_constructor::<TemporalZonedDateTimeConstructor>(
            agent, realm,
        )
        .with_property_capacity(1)
        .with_prototype_property(zoned_date_time_prototype.into_object())
        .build();
    }
}
