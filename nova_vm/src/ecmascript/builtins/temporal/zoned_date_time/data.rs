// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::{
    ecmascript::types::OrdinaryObject,
    heap::{CompactionLists, HeapMarkAndSweep, WorkQueues},
};

#[derive(Debug, Clone, Copy)] // cannot derive copy because of temporal_rs::ZonedDateTime not deriving it 
pub struct ZonedDateTimeHeapData<'a> {
    pub(crate) object_index: Option<OrdinaryObject<'a>>,
    pub(crate) zoned_date_time: temporal_rs::ZonedDateTime,
}

impl ZonedDateTimeHeapData<'_> {
    pub fn default() -> Self {
        todo!()
    }
}

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
