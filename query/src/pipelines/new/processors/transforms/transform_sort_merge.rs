use std::sync::Arc;
use common_datablocks::{DataBlock, SortColumnDescription};
use common_exception::{ErrorCode, Result};
use common_planners::SortPlan;
use crate::pipelines::new::processors::port::{InputPort, OutputPort};
use crate::pipelines::new::processors::Processor;
use crate::pipelines::new::processors::processor::{Event, ProcessorPtr};

pub enum TransformSortMerge {
    Consume(ConsumeState),
    Sorting(SortingBlockState),
    Sorted(SortedState),
    Finished,
}

impl TransformSortMerge {
    pub fn try_create(
        input_port: Arc<InputPort>,
        output_port: Arc<OutputPort>,
        limit: Option<usize>,
        sort_columns_descriptions: Vec<SortColumnDescription>,
    ) -> Result<ProcessorPtr> {
        Ok(ProcessorPtr::create(Box::new(TransformSortMerge::Consume(ConsumeState {
            limit,
            input_port,
            output_port,
            sort_columns_descriptions,
            input_data_blocks: vec![],
        }))))
    }

    #[inline(always)]
    fn to_sorting_state(mut self) -> Result<Self> {
        match self {
            TransformSortMerge::Consume(state) => Ok(TransformSortMerge::Sorting(
                SortingBlockState {
                    input_port: state.input_port,
                    output_port: state.output_port,
                    blocks: state.input_data_blocks,
                    limit: state.limit,
                    sort_columns_descriptions: state.sort_columns_descriptions,
                }
            )),
            _ => Err(ErrorCode::LogicalError("State invalid, must be consume state")),
        }
    }

    #[inline(always)]
    fn to_sorted_state(mut self, sorted_block: Option<DataBlock>) -> Result<Self> {
        match self {
            TransformSortMerge::Sorting(state) => Ok(TransformSortMerge::Sorted(
                SortedState {
                    sorted_block,
                    input_port: state.input_port,
                    output_port: state.output_port,
                }
            )),
            _ => Err(ErrorCode::LogicalError("State invalid, must be sorting state")),
        }
    }

    #[inline(always)]
    fn consume_event(&mut self) -> Result<Event> {
        if let TransformSortMerge::Consume(state) = self {
            if state.input_port.is_finished() {
                let mut temp_state = TransformSortMerge::Finished;
                std::mem::swap(self, &mut temp_state);
                temp_state = temp_state.to_sorting_state()?;
                std::mem::swap(self, &mut temp_state);
                debug_assert!(matches!(temp_state, TransformSortMerge::Finished));
                return Ok(Event::Sync);
            }

            if state.input_port.has_data() {
                state.input_data_blocks.push(state.input_port.pull_data().unwrap()?);
            }

            state.input_port.set_need_data();
            return Ok(Event::NeedData);
        }

        Err(ErrorCode::LogicalError("It's a bug"))
    }
}

#[async_trait::async_trait]
impl Processor for TransformSortMerge {
    fn name(&self) -> &'static str {
        "SortMergeTransform"
    }

    fn event(&mut self) -> Result<Event> {
        match self {
            TransformSortMerge::Finished => Ok(Event::Finished),
            TransformSortMerge::Consume(_) => self.consume_event(),
            TransformSortMerge::Sorting(_) => Err(ErrorCode::LogicalError("It's a bug.")),
            TransformSortMerge::Sorted(state) => {
                if state.output_port.is_finished() {
                    state.input_port.finish();
                    return Ok(Event::Finished);
                }

                if !state.output_port.can_push() {
                    return Ok(Event::NeedConsume);
                }

                match state.sorted_block.take() {
                    None => {
                        state.output_port.finish();
                        Ok(Event::Finished)
                    }
                    Some(data) => {
                        state.output_port.push_data(Ok(data));
                        Ok(Event::NeedConsume)
                    }
                }
            }
        }
    }

    fn process(&mut self) -> Result<()> {
        if let TransformSortMerge::Sorting(state) = self {
            let sorted_block = match state.blocks.is_empty() {
                true => None,
                false => {
                    let desc = &state.sort_columns_descriptions;
                    Some(DataBlock::merge_sort_blocks(&state.blocks, desc, state.limit)?)
                }
            };


            let mut temp_state = TransformSortMerge::Finished;
            std::mem::swap(self, &mut temp_state);
            temp_state = temp_state.to_sorted_state(sorted_block)?;
            std::mem::swap(self, &mut temp_state);
            debug_assert!(matches!(temp_state, TransformSortMerge::Finished));
            return Ok(());
        }

        Err(ErrorCode::LogicalError("State invalid. it's a bug."))
    }
}


pub struct SortedState {
    input_port: Arc<InputPort>,
    output_port: Arc<OutputPort>,
    sorted_block: Option<DataBlock>,
}

pub struct ConsumeState {
    input_port: Arc<InputPort>,
    output_port: Arc<OutputPort>,
    input_data_blocks: Vec<DataBlock>,
    limit: Option<usize>,
    sort_columns_descriptions: Vec<SortColumnDescription>,
}

pub struct SortingBlockState {
    input_port: Arc<InputPort>,
    output_port: Arc<OutputPort>,
    blocks: Vec<DataBlock>,

    limit: Option<usize>,
    sort_columns_descriptions: Vec<SortColumnDescription>,
}
