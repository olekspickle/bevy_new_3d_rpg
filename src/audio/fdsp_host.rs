use super::*;
use bevy_seedling::{
    firewheel::{
        StreamInfo,
        channel_config::ChannelConfig,
        node::{
            AudioNode, AudioNodeInfo, AudioNodeProcessor, ConstructProcessorContext, NodeError,
            ProcBuffers, ProcExtra, ProcInfo, ProcessStatus,
        },
    },
    node::RegisterNode,
};
use firewheel::node::ProcStreamCtx;
use fundsp::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_simple_node::<FundspNode>()
        .add_observer(observe_config_add);
}

fn observe_config_add(trigger: On<Add, FundspConfig>, mut commands: Commands) {
    commands
        .entity(trigger.event_target())
        .insert_if_new(FundspNode);
}

#[derive(Debug, Clone, Component)]
pub struct FundspNode;

#[derive(Clone, Component)]
pub struct FundspConfig {
    pub skip_silence: bool,
    unit: Box<dyn AudioUnit>,
}

impl FundspConfig {
    pub fn _new<U: AudioUnit + 'static>(unit: U) -> Self {
        Self {
            skip_silence: true,
            unit: Box::new(unit),
        }
    }

    /// Create a stereo node from a mono processing chain.
    ///
    /// This simply downmixes the input and splits the output.
    pub fn _new_downmix<U: fundsp::audionode::AudioNode<Inputs = U1, Outputs = U1> + 'static>(
        unit: An<U>,
    ) -> Self {
        let graph = ((pass() + pass()) * 0.5) >> unit >> split::<U2>();

        Self {
            skip_silence: true,
            unit: Box::new(graph),
        }
    }

    pub fn _audio_unit(&self) -> &dyn AudioUnit {
        self.unit.as_ref()
    }

    pub fn _audio_unit_mut(&mut self) -> &mut dyn AudioUnit {
        self.unit.as_mut()
    }

    pub fn _with_skip_silence(self, skip_silence: bool) -> Self {
        Self {
            skip_silence,
            unit: self.unit,
        }
    }
}

impl Default for FundspConfig {
    fn default() -> Self {
        Self {
            skip_silence: true,
            unit: Box::new(pass()),
        }
    }
}

impl PartialEq for FundspConfig {
    fn eq(&self, other: &Self) -> bool {
        self.unit.get_id() == other.unit.get_id() && self.skip_silence == other.skip_silence
    }
}

impl AudioNode for FundspNode {
    type Configuration = FundspConfig;

    fn info(&self, configuration: &Self::Configuration) -> Result<AudioNodeInfo, NodeError> {
        Ok(AudioNodeInfo::new()
            .debug_name("Fun DSP")
            .channel_config(ChannelConfig::new(
                configuration.unit.inputs(),
                configuration.unit.outputs(),
            )))
    }

    fn construct_processor(
        &self,
        configuration: &Self::Configuration,
        cx: ConstructProcessorContext,
    ) -> Result<impl AudioNodeProcessor, NodeError> {
        let mut unit = configuration.clone();
        unit.unit
            .set_sample_rate(cx.stream_info.sample_rate.get() as f64);
        unit.unit.allocate();

        let input_buffer = (0..unit.unit.inputs()).map(|_| 0f32).collect();
        let output_buffer = (0..unit.unit.outputs()).map(|_| 0f32).collect();

        Ok(FundspProcessor {
            unit,
            input_buffer,
            output_buffer,
        })
    }
}

struct FundspProcessor {
    unit: FundspConfig,
    input_buffer: Box<[f32]>,
    output_buffer: Box<[f32]>,
}

impl AudioNodeProcessor for FundspProcessor {
    fn process(
        &mut self,
        info: &ProcInfo,
        ProcBuffers { inputs, outputs }: ProcBuffers,
        _: &mut ProcExtra,
    ) -> ProcessStatus {
        if self.unit.skip_silence
            && info
                .in_silence_mask
                .all_channels_silent(self.unit.unit.inputs())
        {
            return ProcessStatus::ClearAllOutputs;
        }

        for frame in 0..info.frames {
            for (i, input) in self.input_buffer.iter_mut().enumerate() {
                *input = inputs[i][frame];
            }

            self.unit
                .unit
                .tick(&self.input_buffer, &mut self.output_buffer);

            for (i, output) in self.output_buffer.iter().enumerate() {
                outputs[i][frame] = *output;
            }
        }

        ProcessStatus::outputs_modified_with_silence_mask(info.out_silence_mask)
    }

    fn new_stream(&mut self, stream_info: &StreamInfo, _ctx: &mut ProcStreamCtx) {
        if stream_info.sample_rate != stream_info.prev_sample_rate {
            self.unit
                .unit
                .set_sample_rate(stream_info.sample_rate.get() as f64);
            self.unit.unit.allocate();
        }
    }
}
