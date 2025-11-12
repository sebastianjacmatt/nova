// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::ops::{Index, IndexMut};

pub(crate) mod data;
pub mod instant_constructor;
pub mod instant_prototype;

use crate::{
    ecmascript::{
        abstract_operations::{operations_on_objects::get, type_conversion::{PreferredType, to_primitive_object}},
        builtins::{
            ordinary::ordinary_create_from_constructor, temporal::{self, duration::{TemporalDuration, create_temporal_duration, to_temporal_duration}}
        },
        execution::{
            JsResult, ProtoIntrinsics,
            agent::{Agent, ExceptionType},
        },
        types::{
            BUILTIN_STRING_MEMORY, Function, InternalMethods, InternalSlots, IntoFunction, IntoValue, Object, OrdinaryObject, Primitive, PropertyKey, String, UNDEFINED_DISCRIMINANT, Value
        },
    },
    engine::{
        ScopableCollection, context::{Bindable, GcScope, NoGcScope, bindable_handle}, rootable::{HeapRootData, HeapRootRef, Rootable, Scopable}
    },
    heap::{
        CompactionLists, CreateHeapData, Heap, HeapMarkAndSweep, HeapSweepWeakReference,
        WorkQueues, indexes::BaseIndex,
    },
};

use self::data::InstantHeapData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TemporalInstant<'a>(BaseIndex<'a, InstantHeapData<'static>>);

impl TemporalInstant<'_> {
    pub(crate) fn inner_instant(self, agent: &Agent) -> &temporal_rs::Instant {
        &agent[self].instant
    }

    //TODO
    pub(crate) const fn _def() -> Self {
        TemporalInstant(BaseIndex::from_u32_index(0))
    }

    pub(crate) const fn get_index(self) -> usize {
        self.0.into_index()
    }

    /// # Safety
    ///
    /// Should be only called once; reinitialising the value is not allowed.
    unsafe fn set_epoch_nanoseconds(
        self,
        agent: &mut Agent,
        epoch_nanoseconds: temporal_rs::Instant,
    ) {
        agent[self].instant = epoch_nanoseconds;
    }
}

bindable_handle!(TemporalInstant);

impl<'a> From<TemporalInstant<'a>> for Value<'a> {
    fn from(value: TemporalInstant<'a>) -> Self {
        Value::Instant(value)
    }
}
impl<'a> From<TemporalInstant<'a>> for Object<'a> {
    fn from(value: TemporalInstant<'a>) -> Self {
        Object::Instant(value)
    }
}
impl<'a> TryFrom<Value<'a>> for TemporalInstant<'a> {
    type Error = ();

    fn try_from(value: Value<'a>) -> Result<Self, ()> {
        match value {
            Value::Instant(idx) => Ok(idx),
            _ => Err(()),
        }
    }
}
impl<'a> TryFrom<Object<'a>> for TemporalInstant<'a> {
    type Error = ();
    fn try_from(object: Object<'a>) -> Result<Self, ()> {
        match object {
            Object::Instant(idx) => Ok(idx),
            _ => Err(()),
        }
    }
}

impl<'a> InternalSlots<'a> for TemporalInstant<'a> {
    const DEFAULT_PROTOTYPE: ProtoIntrinsics = ProtoIntrinsics::TemporalInstant;
    fn get_backing_object(self, agent: &Agent) -> Option<OrdinaryObject<'static>> {
        agent[self].object_index
    }
    fn set_backing_object(self, agent: &mut Agent, backing_object: OrdinaryObject<'static>) {
        assert!(agent[self].object_index.replace(backing_object).is_none());
    }
}

impl<'a> InternalMethods<'a> for TemporalInstant<'a> {}

// TODO: get rid of Index impls, replace with get/get_mut/get_direct/get_direct_mut functions
impl Index<TemporalInstant<'_>> for Agent {
    type Output = InstantHeapData<'static>;

    fn index(&self, index: TemporalInstant<'_>) -> &Self::Output {
        &self.heap.instants[index]
    }
}

impl IndexMut<TemporalInstant<'_>> for Agent {
    fn index_mut(&mut self, index: TemporalInstant) -> &mut Self::Output {
        &mut self.heap.instants[index]
    }
}

impl Index<TemporalInstant<'_>> for Vec<InstantHeapData<'static>> {
    type Output = InstantHeapData<'static>;

    fn index(&self, index: TemporalInstant<'_>) -> &Self::Output {
        self.get(index.get_index())
            .expect("heap access out of bounds")
    }
}

impl IndexMut<TemporalInstant<'_>> for Vec<InstantHeapData<'static>> {
    fn index_mut(&mut self, index: TemporalInstant<'_>) -> &mut Self::Output {
        self.get_mut(index.get_index())
            .expect("heap access out of bounds")
    }
}

impl Rootable for TemporalInstant<'_> {
    type RootRepr = HeapRootRef;

    fn to_root_repr(value: Self) -> Result<Self::RootRepr, HeapRootData> {
        Err(HeapRootData::Instant(value.unbind()))
    }

    fn from_root_repr(value: &Self::RootRepr) -> Result<Self, HeapRootRef> {
        Err(*value)
    }

    fn from_heap_ref(heap_ref: HeapRootRef) -> Self::RootRepr {
        heap_ref
    }

    fn from_heap_data(heap_data: HeapRootData) -> Option<Self> {
        match heap_data {
            HeapRootData::Instant(object) => Some(object),
            _ => None,
        }
    }
}

impl HeapMarkAndSweep for TemporalInstant<'static> {
    fn mark_values(&self, queues: &mut WorkQueues) {
        queues.instants.push(*self);
    }
    fn sweep_values(&mut self, compactions: &CompactionLists) {
        compactions.instants.shift_index(&mut self.0);
    }
}

impl HeapSweepWeakReference for TemporalInstant<'static> {
    fn sweep_weak_reference(self, compactions: &CompactionLists) -> Option<Self> {
        compactions.instants.shift_weak_index(self.0).map(Self)
    }
}

impl<'a> CreateHeapData<InstantHeapData<'a>, TemporalInstant<'a>> for Heap {
    fn create(&mut self, data: InstantHeapData<'a>) -> TemporalInstant<'a> {
        self.instants.push(data.unbind());
        self.alloc_counter += core::mem::size_of::<InstantHeapData<'static>>();
        TemporalInstant(BaseIndex::last_t(&self.instants))
    }
}

/// 8.5.2 CreateTemporalInstant ( epochNanoseconds [ , newTarget ] )
///
/// The abstract operation CreateTemporalInstant takes argument
/// epochNanoseconds (a BigInt) and optional argument newTarget (a constructor)
/// and returns either a normal completion containing a Temporal.Instant or a
/// throw completion. It creates a Temporal.Instant instance and fills the
/// internal slots with valid values.
fn create_temporal_instant<'gc>(
    agent: &mut Agent,
    epoch_nanoseconds: temporal_rs::Instant,
    new_target: Option<Function>,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, TemporalInstant<'gc>> {
    // 1. Assert: IsValidEpochNanoseconds(epochNanoseconds) is true.
    // 2. If newTarget is not present, set newTarget to %Temporal.Instant%.
    let new_target = new_target.unwrap_or_else(|| {
        agent
            .current_realm_record()
            .intrinsics()
            .temporal_instant()
            .into_function()
    });
    // 3. Let object be ? OrdinaryCreateFromConstructor(newTarget, "%Temporal.Instant.prototype%", « [[InitializedTemporalInstant]], [[EpochNanoseconds]] »).
    let Object::Instant(object) =
        ordinary_create_from_constructor(agent, new_target, ProtoIntrinsics::TemporalInstant, gc)?
    else {
        unreachable!()
    };
    // 4. Set object.[[EpochNanoseconds]] to epochNanoseconds.
    // SAFETY: initialising Instant.
    unsafe { object.set_epoch_nanoseconds(agent, epoch_nanoseconds) };
    // 5. Return object.
    Ok(object)
}

/// ### [8.5.3 ToTemporalInstant ( item )](https://tc39.es/proposal-temporal/#sec-temporal-totemporalinstant)
///
/// The abstract operation ToTemporalInstant takes argument item (an ECMAScript language value) and
/// returns either a normal completion containing a Temporal.Instant or a throw completion.
/// Converts item to a new Temporal.Instant instance if possible, and throws otherwise.
fn to_temporal_instant<'gc>(
    agent: &mut Agent,
    item: Value,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, temporal_rs::Instant> {
    let item = item.bind(gc.nogc());
    // 1. If item is an Object, then
    let item = if let Ok(item) = Object::try_from(item) {
        // a. If item has an [[InitializedTemporalInstant]] or [[InitializedTemporalZonedDateTime]]
        // internal slot, then TODO: TemporalZonedDateTime::try_from(item)
        if let Ok(item) = TemporalInstant::try_from(item) {
            // i. Return ! CreateTemporalInstant(item.[[EpochNanoseconds]]).
            return Ok(agent[item].instant);
        }
        // b. NOTE: This use of ToPrimitive allows Instant-like objects to be converted.
        // c. Set item to ? ToPrimitive(item, string).
        to_primitive_object(agent, item.unbind(), Some(PreferredType::String), gc)?
    } else {
        Primitive::try_from(item).unwrap()
    };
    // 2. If item is not a String, throw a TypeError exception.
    let Ok(item) = String::try_from(item) else {
        todo!() // TypeErrror
    };
    // 3. Let parsed be ? ParseISODateTime(item, « TemporalInstantString »).
    // 4. Assert: Either parsed.[[TimeZone]].[[OffsetString]] is not empty or
    //    parsed.[[TimeZone]].[[Z]] is true, but not both.
    // 5. If parsed.[[TimeZone]].[[Z]] is true, let offsetNanoseconds be 0; otherwise, let
    //    offsetNanoseconds be ! ParseDateTimeUTCOffset(parsed.[[TimeZone]].[[OffsetString]]).
    // 6. If parsed.[[Time]] is start-of-day, let time be MidnightTimeRecord(); else let time be
    //    parsed.[[Time]].
    // 7. Let balanced be BalanceISODateTime(parsed.[[Year]], parsed.[[Month]], parsed.[[Day]],
    //    time.[[Hour]], time.[[Minute]], time.[[Second]], time.[[Millisecond]],
    //    time.[[Microsecond]], time.[[Nanosecond]] - offsetNanoseconds).
    // 8. Perform ? CheckISODaysRange(balanced.[[ISODate]]).
    // 9. Let epochNanoseconds be GetUTCEpochNanoseconds(balanced).
    // 10. If IsValidEpochNanoseconds(epochNanoseconds) is false, throw a RangeError exception.
    // 11. Return ! CreateTemporalInstant(epochNanoseconds).
    let parsed = temporal_rs::Instant::from_utf8(item.as_bytes(agent)).unwrap();
    Ok(parsed)
}

/// [8.5.10 AddDurationToInstant ( operation, instant, temporalDurationLike )](https://tc39.es/proposal-temporal/#sec-temporal-adddurationtoinstant)
/// The abstract operation AddDurationToInstant takes arguments operation
/// (add or subtract), instant (a Temporal.Instant),
/// and temporalDurationLike (an ECMAScript language value)
/// and returns either a normal completion containing a Temporal.Instant
/// or a throw completion.
/// It adds/subtracts temporalDurationLike to/from instant.
fn add_duration_to_instant<'gc, const IS_ADD: bool>(
    agent: &mut Agent,
    instant: TemporalInstant,
    duration: Value,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    let duration = duration.bind(gc.nogc());
    let instant = instant.bind(gc.nogc());
    // 1. Let duration be ? ToTemporalDuration(temporalDurationLike).
    let instant = instant.scope(agent, gc.nogc());
    let duration = to_temporal_duration(agent, duration.unbind(), gc.reborrow());
    // 2. If operation is subtract, set duration to CreateNegatedTemporalDuration(duration).
    // 3. Let largestUnit be DefaultTemporalLargestUnit(duration).
    // 4. If TemporalUnitCategory(largestUnit) is date, throw a RangeError exception.
    // 5. Let internalDuration be ToInternalDurationRecordWith24HourDays(duration).
    // 6. Let ns be ? AddInstant(instant.[[EpochNanoseconds]], internalDuration.[[Time]]).
    let ns_result = if IS_ADD {
        temporal_rs::Instant::add(&agent[instant.get(agent)].instant, &duration.unwrap()).unwrap()
    } else {
        temporal_rs::Instant::subtract(&agent[instant.get(agent)].instant, &duration.unwrap())
            .unwrap()
    };
    // 7. Return ! CreateTemporalInstant(ns).
    let instant = create_temporal_instant(agent, ns_result, None, gc)?;
    Ok(instant.into_value())
}

/// [8.5.9 DifferenceTemporalInstant ( operation, instant, other, options )](https://tc39.es/proposal-temporal/#sec-temporal-differencetemporalinstant)
/// The abstract operation DifferenceTemporalInstant takes arguments
/// operation (since or until), instant (a Temporal.Instant),
/// other (an ECMAScript language value), and options
/// (an ECMAScript language value) and returns either
/// a normal completion containing a Temporal.Duration or a
/// throw completion. It computes the difference between the
/// two times represented by instant and other, optionally
/// rounds it, and returns it as a Temporal.Duration object.
fn difference_temporal_instant<'gc, const IS_UNTIL: bool>(
    agent: &mut Agent,
    instant: TemporalInstant,
    other: Value,
    options: Value,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, TemporalDuration<'gc>> {
    let instant = instant.scope(agent, gc.nogc());
    let other = other.bind(gc.nogc());
    let options = options.scope(agent, gc.nogc());
    // 1. Set other to ? ToTemporalInstant(other).
    let other = to_temporal_instant(agent, other.unbind(), gc.reborrow())
        .unbind()?
        .bind(gc.nogc());
    // 2. Let resolvedOptions be ? GetOptionsObject(options).
    let resolved_options = get_options_object(agent, options.get(agent).unbind(), gc.reborrow())
        .unbind()?
        .bind(gc.nogc());    
    // 3. Let settings be ? GetDifferenceSettings(operation, resolvedOptions, time, « », nanosecond, second).
    let settings = get_difference_settings::<IS_UNTIL>(
        agent,
        resolved_options.unbind(),
        temporal_rs::options::UnitGroup::Time,
        vec![],
        temporal_rs::options::Unit::Nanosecond,
        temporal_rs::options::Unit::Second,
        gc,
    )?;
    // 4. Let internalDuration be DifferenceInstant(instant.[[EpochNanoseconds]], other.[[EpochNanoseconds]], settings.[[RoundingIncrement]], settings.[[SmallestUnit]], settings.[[RoundingMode]]).
    // 5. Let result be ! TemporalDurationFromInternal(internalDuration, settings.[[LargestUnit]]).
    // 6. If operation is since, set result to CreateNegatedTemporalDuration(result).
    // 7. Return result.
    let result = if IS_UNTIL {
        temporal_rs::Instant::until(&agent[instant.get(agent)].instant, &other, settings).unwrap()
    } else {
        temporal_rs::Instant::since(&agent[instant.get(agent)].instant, &other, settings).unwrap()
    };
    create_temporal_duration() // todo: impl create_temporal_duration and use result values
}

// <------ TODO: Abstract Operation unrelated to instant, preferably move these into a seperate temporal/abstract_operations/ ------>

/// [13.17 GetTemporalUnitValuedOption ( options, key, default )](https://tc39.es/proposal-temporal/#sec-temporal-gettemporalunitvaluedoption)
/// The abstract operation GetTemporalUnitValuedOption takes arguments options 
/// (an Object), key (a property key), and default (required or unset) and 
/// returns either a normal completion containing either a Temporal unit, 
/// unset, or auto, or a throw completion. 
/// It attempts to read a Temporal unit from the specified property of options.
/// Both singular and plural unit names are accepted, but only the singular form is used internally.
pub (crate) fn get_temporal_unit_valued_option<'gc, const REQUIRED: bool> (
    agent: &mut Agent,
    options: Object<'gc>,
    property_key: PropertyKey,
    gc: GcScope<'gc, '_>
) -> JsResult<'gc, temporal_rs::options::Unit> {
    let options = options.bind(gc.nogc());
    // 1. Let allowedStrings be a List containing all values in the "Singular property name" and "Plural property name" columns of Table 21, except the header row.
    let mut allowed_strings = Vec::<Value>::with_capacity(21).scope(agent, gc.nogc());
    for s in [
        BUILTIN_STRING_MEMORY.year,
        BUILTIN_STRING_MEMORY.years,
        BUILTIN_STRING_MEMORY.month,
        BUILTIN_STRING_MEMORY.months,
        BUILTIN_STRING_MEMORY.week,
        BUILTIN_STRING_MEMORY.weeks,
        BUILTIN_STRING_MEMORY.day,
        BUILTIN_STRING_MEMORY.days,
        BUILTIN_STRING_MEMORY.hour,
        BUILTIN_STRING_MEMORY.hours,
        BUILTIN_STRING_MEMORY.minute,
        BUILTIN_STRING_MEMORY.minutes,
        BUILTIN_STRING_MEMORY.second,
        BUILTIN_STRING_MEMORY.seconds,
        BUILTIN_STRING_MEMORY.millisecond,
        BUILTIN_STRING_MEMORY.milliseconds,
        BUILTIN_STRING_MEMORY.microsecond,
        BUILTIN_STRING_MEMORY.microseconds,
        BUILTIN_STRING_MEMORY.nanosecond,
        BUILTIN_STRING_MEMORY.nanoseconds,
        BUILTIN_STRING_MEMORY.auto,
    ] {
        allowed_strings.push(agent,s.to_property_key());
    }
    let allowed_vec = allowed_strings.take(agent);

    // 2. Append "auto" to allowedStrings.
    // 3. NOTE: For each singular Temporal unit name that is contained within allowedStrings, the corresponding plural name is also contained within it.
    // 4. If default is unset, then
    let default : Value = if !REQUIRED {
        // a. Let defaultValue be undefined.
        Value::Undefined
    } else {
        // 5. Else,
        // a. Let defaultValue be default.
        Value::default()
    };
    // 6. Let value be ? GetOption(options, key, string, allowedStrings, defaultValue).
    let value = get_option(
        agent,
        options.unbind(),
        property_key,
        true,
        allowed_vec,
        default,
        gc.reborrow(),
    )?.bind(gc.nogc());
    // 7. If value is undefined, return unset.
    if value.is_undefined() {
        todo!()
        //return temporal_rs::options::Unit:: // what is unset?
    }
    // 8. If value is "auto", return auto.
    if value.to_string(agent, gc)?.as_str(agent).unwrap().eq("auto") {
        return  Ok(temporal_rs::options::Unit::Auto);
    }
    // 9. Return the value in the "Value" column of Table 21 corresponding to the row with value in its "Singular property name" or "Plural property name" column.
    
    unimplemented!()
}


/// [14.5.2.2 GetOption ( options, property, type, values, default )](https://tc39.es/proposal-temporal/#sec-getoption)
/// The abstract operation GetOption takes arguments options (an Object),
/// property (a property key), type (boolean or string),
/// values (empty or a List of ECMAScript language values),
/// and default (required or an ECMAScript language value)
/// and returns either a normal completion containing an
/// ECMAScript language value or a throw completion.
/// It extracts the value of the specified property of options,
/// converts it to the required type, 
/// checks whether it is allowed by values if values is not empty,
/// and substitutes default if the value is undefined.
pub (crate) fn get_option<'gc> (
    agent: &mut Agent,
    options: Object<'gc>,
    property_key: PropertyKey,
    types: bool,
    values: Vec<Value<'gc>>,
    default: Value<'gc>,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Value<'gc>> {
    // 1. Let value be ? Get(options, property).
    // 2. If value is undefined, then
    //    a. If default is required, throw a RangeError exception.
    //    b. Return default.
    // 3. If type is boolean, then
    //    a. Set value to ToBoolean(value).
    // 4. Else,
    //    a. Assert: type is string.
    //    b. Set value to ? ToString(value).
    // 5. If values is not empty and values does not contain value, throw a RangeError exception.
    // 6. Return value.
    unimplemented!()
}


/// [13.42 GetDifferenceSettings ( operation, options, unitGroup, disallowedUnits, fallbackSmallestUnit, smallestLargestDefaultUnit )](https://tc39.es/proposal-temporal/#sec-temporal-getdifferencesettings)
/// The abstract operation GetDifferenceSettings takes arguments operation (since or until),
/// options (an Object), unitGroup (date, time, or datetime), disallowedUnits (a List of Temporal units),
/// fallbackSmallestUnit (a Temporal unit), and smallestLargestDefaultUnit (a Temporal unit) and returns either
/// a normal completion containing a Record with fields [[SmallestUnit]] (a Temporal unit),
/// [[LargestUnit]] (a Temporal unit), [[RoundingMode]] (a rounding mode),
/// and [[RoundingIncrement]] (an integer in the inclusive interval from 1 to 10**9), 
/// or a throw completion. It reads unit and rounding options needed by difference operations. 
pub(crate) fn get_difference_settings<'gc, const IS_UNTIL: bool> (
    agent: &mut Agent,
    options: Object<'gc>, // options (an Object)
    unit_group: temporal_rs::options::UnitGroup, // unitGroup (date, time, or datetime)
    disallowed_units: Vec<temporal_rs::options::Unit>, // disallowedUnits (todo:a List of Temporal units)
    fallback_smallest_unit: temporal_rs::options::Unit, // fallbackSmallestUnit (a Temporal unit) 
    smallest_largest_default_unit: temporal_rs::options::Unit, // smallestLargestDefaultUnit (a Temporal unit)
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, temporal_rs::options::DifferenceSettings> {
    let options = options.bind(gc.nogc());
    // 1. NOTE: The following steps read options and perform independent validation in alphabetical order.
    // 2. Let largestUnit be ? GetTemporalUnitValuedOption(options, "largestUnit", unset).
    const UNSET:bool = false;
    let largest_unit = get_temporal_unit_valued_option::<UNSET>(agent, options.unbind(),BUILTIN_STRING_MEMORY.largestUnit.to_property_key(), gc);
    // 3. Let roundingIncrement be ? GetRoundingIncrementOption(options).
    let rounding_increment = get_rounding_increment_option(agent, options.unbind(), gc);
    // 4. Let roundingMode be ? GetRoundingModeOption(options, trunc).
    let rounding_mode = get_rounding_mode_option(agent, options.unbind(),temporal_rs::options::RoundingMode::Trunc, gc);
    // 5. Let smallestUnit be ? GetTemporalUnitValuedOption(options, "smallestUnit", unset).
    let smallest_unit = get_temporal_unit_valued_option::<UNSET>(agent, options.unbind(), BUILTIN_STRING_MEMORY.smallestUnit.to_property_key(), gc);
    // 6. Perform ? ValidateTemporalUnitValue(largestUnit, unitGroup, « auto »).
    // 7. If largestUnit is unset, then
    //    a. Set largestUnit to auto.
    // 8. If disallowedUnits contains largestUnit, throw a RangeError exception.
    // 9. If operation is since, then
    //    a. Set roundingMode to NegateRoundingMode(roundingMode).
    // 10. Perform ? ValidateTemporalUnitValue(smallestUnit, unitGroup).
    // 11. If smallestUnit is unset, then
    //     a. Set smallestUnit to fallbackSmallestUnit.
    // 12. If disallowedUnits contains smallestUnit, throw a RangeError exception.
    // 13. Let defaultLargestUnit be LargerOfTwoTemporalUnits(smallestLargestDefaultUnit, smallestUnit).
    // 14. If largestUnit is auto, set largestUnit to defaultLargestUnit.
    // 15. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
    // 16. Let maximum be MaximumTemporalDurationRoundingIncrement(smallestUnit).
    // 17. If maximum is not unset, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
    // 18. Return the Record { [[SmallestUnit]]: smallestUnit, [[LargestUnit]]: largestUnit, [[RoundingMode]]: roundingMode, [[RoundingIncrement]]: roundingIncrement,  }.
    let mut diff_settings = temporal_rs::options::DifferenceSettings::default();
    diff_settings.largest_unit = Some(largest_unit.unwrap());
    diff_settings.smallest_unit = Some(smallest_unit.unwrap());
    diff_settings.rounding_mode = Some(rounding_mode.unwrap());
    diff_settings.increment = Some(rounding_increment.unwrap());
    Ok(diff_settings)
}

/// [14.5.2.1 GetOptionsObject ( options )](https://tc39.es/proposal-temporal/#sec-getoptionsobject)
/// The abstract operation GetOptionsObject takes argument options (an ECMAScript language value) 
/// and returns either a normal completion containing an Object or a throw completion. 
/// It returns an Object suitable for use with GetOption, 
/// either options itself or a default empty Object. 
/// It throws a TypeError if options is not undefined and not an Object.
pub (crate) fn get_options_object<'gc> (
    agent: &mut Agent,
    options: Value, // options (an ECMAScript language value)
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, Object<'gc>> {
    let options = options.bind(gc.nogc());
    // 1. If options is undefined, then
    if options.unbind().is_undefined().bind(gc.nogc()) {
        // a. Return OrdinaryObjectCreate(null).
        let obj = OrdinaryObject::create_empty_object(agent, gc.nogc()).bind(gc.nogc());
        return Ok(Object::Object(obj.unbind()));
    }
    // 2. If options is an Object, then
    if options.unbind().is_object().bind(gc.nogc()) {
        // a. Return options itself.
        let obj = Object::try_from(options.unbind()).bind(gc.nogc());
        return Ok(obj.unwrap().unbind());
    }
    // 3. Throw a TypeError exception.
    Err(agent.throw_exception_with_static_message(
        ExceptionType::TypeError,
        "options is not undefined and not an Object",
        gc.into_nogc(),
    ))
}

/// [14.5.2.3 GetRoundingModeOption ( options, fallback )](https://tc39.es/proposal-temporal/#sec-temporal-getroundingmodeoption)
/// The abstract operation GetRoundingModeOption takes arguments options (an Object)
/// and fallback (a rounding mode) and returns either a normal completion containing a rounding mode,
/// or a throw completion. It fetches and validates the "roundingMode" property from options,
/// returning fallback as a default if absent.
pub (crate) fn get_rounding_mode_option<'gc> (
    agent: &mut Agent,
    options: Object,
    fallback: temporal_rs::options::RoundingMode,
    gc: GcScope<'gc, '_>,
) -> JsResult<'gc, temporal_rs::options::RoundingMode> {
    // 1. Let allowedStrings be the List of Strings from the "String Identifier" column of Table 28.
    // 2. Let stringFallback be the value from the "String Identifier" column of the row with fallback in its "Rounding Mode" column.
    // 3. Let stringValue be ? GetOption(options, "roundingMode", string, allowedStrings, stringFallback).
    // 4. Return the value from the "Rounding Mode" column of the row with stringValue in its "String Identifier" column.
    // A rounding mode is one of the values in the "Rounding Mode" column of Table 28. An unsigned rounding mode is one of the values in the "Unsigned Rounding Mode" column of Table 22.
    unimplemented!()
}

/// [14.5.2.4 GetRoundingIncrementOption ( options )](https://tc39.es/proposal-temporal/#sec-temporal-getroundingincrementoption)
/// The abstract operation GetRoundingIncrementOption takes argument options (an Object) and returns
/// either a normal completion containing a positive integer in the inclusive interval from 1 to 10**9,
/// or a throw completion. It fetches and validates the "roundingIncrement" property from options,
/// returning a default if absent.
pub (crate) fn get_rounding_increment_option<'gc> (
    agent: &mut Agent,
    options: Object, // options (an Object)
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, temporal_rs::options::RoundingIncrement> {
    // 1. Let value be ? Get(options, "roundingIncrement").
    let value = get(agent, options, BUILTIN_STRING_MEMORY.roundingIncrement.to_property_key(), gc.reborrow())
        .unbind()?
        .bind(gc.nogc());
    // 2. If value is undefined, return 1𝔽.
    if value.unbind().is_undefined() {
        return Ok(temporal_rs::options::RoundingIncrement::ONE);
    }
    // 3. Let integerIncrement be ? ToIntegerWithTruncation(value).
    let integer_increment = to_integer_with_truncation(agent, value.unbind(), gc.reborrow())
        .unbind()?
        .bind(gc.nogc());
    // 4. If integerIncrement < 1 or integerIncrement > 10**9, throw a RangeError exception.
    if integer_increment < 1 || integer_increment > 1_000_000_000 { // todo, define this value properly in rust
        return Err(agent.throw_exception_with_static_message(
            ExceptionType::RangeError,
            "integerIncrement < 1 or integerIncrement > 10**9",
            gc.into_nogc(),
        ));
    }
    // 5. Return integerIncrement.
    Ok(temporal_rs::options::RoundingIncrement::try_new(integer_increment as u32).unwrap())
}

/// [13.40 ToIntegerWithTruncation ( argument )](https://tc39.es/proposal-temporal/#sec-tointegerwithtruncation)
/// The abstract operation ToIntegerWithTruncation takes argument argument (an ECMAScript language value)
/// and returns either a normal completion containing an integer or a throw completion. 
/// It converts argument to an integer representing its Number value with fractional part truncated, 
/// or throws a RangeError when that value is not finite.
pub (crate) fn to_integer_with_truncation<'gc> (
    agent: &mut Agent,
    argument: Value,
    mut gc: GcScope<'gc, '_>,
) -> JsResult<'gc, i64> {
    let argument = argument.bind(gc.nogc());
    // 1. Let number be ? ToNumber(argument).
    let number = argument.unbind().to_number(agent, gc.reborrow())
        .unbind()?
        .bind(gc.nogc());
    // 2. If number is NaN, +∞𝔽 or -∞𝔽, throw a RangeError exception.
    if number.is_nan(agent) || number.is_pos_infinity(agent) || number.is_neg_infinity(agent) {
        return Err(agent.throw_exception_with_static_message(
            ExceptionType::RangeError,
            "number is NaN, +∞𝔽 or -∞𝔽",
            gc.into_nogc(),
        ));
    }
    // 3. Return truncate(ℝ(number)).
    Ok(number.unbind().truncate(agent,gc.nogc()).into_i64(agent))
}




#[inline(always)]
fn require_internal_slot_temporal_instant<'a>(
    agent: &mut Agent,
    value: Value,
    gc: NoGcScope<'a, '_>,
) -> JsResult<'a, TemporalInstant<'a>> {
    match value {
        Value::Instant(instant) => Ok(instant.bind(gc)),
        _ => Err(agent.throw_exception_with_static_message(
            ExceptionType::TypeError,
            "Object is not a Temporal Instant",
            gc,
        )),
    }
}
