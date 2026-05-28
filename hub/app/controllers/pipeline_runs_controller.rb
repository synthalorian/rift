class PipelineRunsController < ApplicationController
  def index
    @title = "Pipeline Runs"
    @runs = RiftDb.recent_runs(limit: 50)
  rescue SQLite3::Exception => e
    @error = "Cannot read Rift database: #{e.message}"
    @runs = []
  end

  def show
    @run = RiftDb.find_run(params[:id])
    if @run.nil?
      redirect_to pipeline_runs_path, alert: "Run not found"
      return
    end
    @errors = RiftDb.errors_for_run(params[:id], limit: 100)
    @title = "Run #{@run["id"].to_s[0..12]}…"
  rescue SQLite3::Exception => e
    @error = "Cannot read Rift database: #{e.message}"
  end
end
