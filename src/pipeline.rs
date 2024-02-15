pub struct PipelineObject {
    pub pipeline: wgpu::RenderPipeline,
    pub polygon_mode: wgpu::PolygonMode,
}

pub struct PipelineManager {
    pub pipelines: Vec<PipelineObject>,
}

impl PipelineManager {
    pub fn add_raw(&mut self, pipeline: wgpu::RenderPipeline, polygon_mode: wgpu::PolygonMode) {
        self.pipelines.push(PipelineObject{ pipeline, polygon_mode });
    }

    pub fn add(&mut self, pipeline: PipelineObject) {
        self.pipelines.push(pipeline);
    }

    pub fn add_multiple(&mut self, pipelines: Vec<PipelineObject>) {
        for pipeline in pipelines {
            self.add(pipeline);
        }
    }

    pub fn first_polygon_fill_item(&self) -> &wgpu::RenderPipeline {
        for pipeline in &self.pipelines {
            if pipeline.polygon_mode == wgpu::PolygonMode::Fill {
                return &pipeline.pipeline;
            }
        }

        &self.pipelines[0].pipeline
    }

    pub fn first_polygon_line_item(&self) -> &wgpu::RenderPipeline {
        for pipeline in &self.pipelines {
            if pipeline.polygon_mode == wgpu::PolygonMode::Line {
                return &pipeline.pipeline;
            }
        }

        &self.pipelines[0].pipeline
    }
}
