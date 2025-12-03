use std::ops::{Index, IndexMut};

use crate::{
    ecmascript::{
        builtins::temporal::zoned_date_time::data::ZonedDateTimeHeapData,
        execution::{Agent, ProtoIntrinsics},
        types::{InternalMethods, InternalSlots, Object, OrdinaryObject, Value},
    },
    engine::{
        context::{Bindable, bindable_handle},
        rootable::{HeapRootData, HeapRootRef, Rootable},
    },
    heap::{
        CompactionLists, CreateHeapData, Heap, HeapMarkAndSweep, HeapSweepWeakReference,
        WorkQueues, indexes::BaseIndex,
    },
};

pub(crate) mod data;
pub mod zoned_date_time_constructor;
pub mod zoned_date_time_prototype;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TemporalZonedDateTime<'a>(BaseIndex<'a, ZonedDateTimeHeapData<'static>>);

impl TemporalZonedDateTime<'_> {
    pub(crate) const fn _def() -> Self {
        TemporalZonedDateTime(BaseIndex::from_u32_index(0))
    }

    pub(crate) const fn get_index(self) -> usize {
        self.0.into_index()
    }
}

bindable_handle!(TemporalZonedDateTime);

impl<'a> From<TemporalZonedDateTime<'a>> for Value<'a> {
    fn from(value: TemporalZonedDateTime<'a>) -> Self {
        Value::ZonedDateTime(value)
    }
}
impl<'a> From<TemporalZonedDateTime<'a>> for Object<'a> {
    fn from(value: TemporalZonedDateTime<'a>) -> Self {
        Object::ZonedDateTime(value)
    }
}
impl<'a> TryFrom<Value<'a>> for TemporalZonedDateTime<'a> {
    type Error = ();

    fn try_from(value: Value<'a>) -> Result<Self, ()> {
        match value {
            Value::ZonedDateTime(idx) => Ok(idx),
            _ => Err(()),
        }
    }
}
impl<'a> TryFrom<Object<'a>> for TemporalZonedDateTime<'a> {
    type Error = ();

    fn try_from(object: Object<'a>) -> Result<Self, ()> {
        match object {
            Object::ZonedDateTime(idx) => Ok(idx),
            _ => Err(()),
        }
    }
}

impl<'a> InternalSlots<'a> for TemporalZonedDateTime<'a> {
    const DEFAULT_PROTOTYPE: ProtoIntrinsics = ProtoIntrinsics::TemporalZonedDateTime;
    fn get_backing_object(self, agent: &Agent) -> Option<OrdinaryObject<'static>> {
        agent[self].object_index
    }
    fn set_backing_object(self, agent: &mut Agent, backing_object: OrdinaryObject<'static>) {
        assert!(agent[self].object_index.replace(backing_object).is_none());
    }
}

impl<'a> InternalMethods<'a> for TemporalZonedDateTime<'a> {}

// TODO: get rid of Index impls, replace with get/get_mut/get_direct/get_direct_mut functions
impl Index<TemporalZonedDateTime<'_>> for Agent {
    type Output = ZonedDateTimeHeapData<'static>;

    fn index(&self, index: TemporalZonedDateTime<'_>) -> &Self::Output {
        &self.heap.zoned_date_times[index]
    }
}

impl IndexMut<TemporalZonedDateTime<'_>> for Agent {
    fn index_mut(&mut self, index: TemporalZonedDateTime<'_>) -> &mut Self::Output {
        &mut self.heap.zoned_date_times[index]
    }
}

impl Index<TemporalZonedDateTime<'_>> for Vec<ZonedDateTimeHeapData<'static>> {
    type Output = ZonedDateTimeHeapData<'static>;

    fn index(&self, index: TemporalZonedDateTime<'_>) -> &Self::Output {
        self.get(index.get_index())
            .expect("heap acess out of bounds")
    }
}

impl IndexMut<TemporalZonedDateTime<'_>> for Vec<ZonedDateTimeHeapData<'static>> {
    fn index_mut(&mut self, index: TemporalZonedDateTime<'_>) -> &mut Self::Output {
        self.get_mut(index.get_index())
            .expect("heap access out of bounds")
    }
}

impl Rootable for TemporalZonedDateTime<'_> {
    type RootRepr = HeapRootRef;

    fn to_root_repr(value: Self) -> Result<Self::RootRepr, HeapRootData> {
        Err(HeapRootData::ZonedDateTime(value.unbind()))
    }

    fn from_root_repr(value: &Self::RootRepr) -> Result<Self, HeapRootRef> {
        Err(*value)
    }

    fn from_heap_ref(heap_ref: HeapRootRef) -> Self::RootRepr {
        heap_ref
    }

    fn from_heap_data(heap_data: HeapRootData) -> Option<Self> {
        match heap_data {
            HeapRootData::ZonedDateTime(object) => Some(object),
            _ => None,
        }
    }
}

impl HeapMarkAndSweep for TemporalZonedDateTime<'static> {
    fn mark_values(&self, queues: &mut WorkQueues) {
        queues.zoned_date_times.push(*self);
    }

    fn sweep_values(&mut self, compactions: &CompactionLists) {
        compactions.zoned_date_times.shift_index(&mut self.0);
    }
}

impl HeapSweepWeakReference for TemporalZonedDateTime<'static> {
    fn sweep_weak_reference(self, compactions: &CompactionLists) -> Option<Self> {
        compactions
            .zoned_date_times
            .shift_weak_index(self.0)
            .map(Self)
    }
}

impl<'a> CreateHeapData<ZonedDateTimeHeapData<'a>, TemporalZonedDateTime<'a>> for Heap {
    fn create(&mut self, data: ZonedDateTimeHeapData<'a>) -> TemporalZonedDateTime<'a> {
        self.zoned_date_times.push(data.unbind()); // temporal_rs::ZonedDateTime does not derive Copy and therefore cannot be bind/unbind, I think it has to be 'static because it is not thread safe.
        self.alloc_counter += core::mem::size_of::<ZonedDateTimeHeapData<'static>>();
        TemporalZonedDateTime(BaseIndex::last(&self.zoned_date_times))
    }
}
