// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::engine::context::NoGcScope;
use crate::{
    ecmascript::types::OrdinaryObject, engine::context::{bindable_handle, trivially_bindable}, heap::{CompactionLists, HeapMarkAndSweep, WorkQueues}
};

#[derive(Debug, Clone)] // cannot derive copy because of temporal_rs::ZonedDateTime not deriving it 
pub struct ZonedDateTimeHeapData<'a> {
    pub(crate) object_index: Option<OrdinaryObject<'a>>,
    pub(crate) zoned_date_time: temporal_rs::ZonedDateTime,
}

impl ZonedDateTimeHeapData<'_> {
    pub fn default() -> Self {
        todo!()
    }
}

trivially_bindable!(temporal_rs::ZonedDateTime);
bindable_handle!(ZonedDateTimeHeapData);

impl HeapMarkAndSweep for ZonedDateTimeHeapData<'static> {
    fn mark_values(&self, queues: &mut WorkQueues) {
        let Self {
            object_index,
            zoned_date_time: _,
        } = self;

        object_index.mark_values(queues);
    }
    fn sweep_values(&mut self, compactions: &CompactionLists) {
        let Self {
            object_index,
            zoned_date_time: _,
        } = self;

        object_index.sweep_values(compactions);
    }
}
