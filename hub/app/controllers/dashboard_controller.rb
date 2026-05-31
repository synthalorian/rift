class DashboardController < ApplicationController
  def index
    @title = "Rift Dashboard"
    @total_assets = RiftDb.total_assets
    @counts = RiftDb.asset_counts
    @recent_runs = RiftDb.recent_runs(limit: 5)
    @last_run = RiftDb.last_run

    respond_to do |format|
      format.html
      format.json {
        render json: {
          total_assets: @total_assets,
          counts: @counts,
          last_run: @last_run,
          recent_runs: @recent_runs,
          ok: @counts["ok"] || 0,
          pending: @counts["pending"] || 0,
          errors: @counts["error"] || 0
        }
      }
    end
  rescue SQLite3::Exception => e
    @error = "Cannot connect to Rift database: #{e.message}"
    @total_assets = 0
    @counts = {"ok" => 0, "pending" => 0, "error" => 0}
    @recent_runs = []
    @last_run = nil

    respond_to do |format|
      format.html
      format.json {
        render json: {
          total_assets: 0,
          counts: @counts,
          last_run: nil,
          recent_runs: [],
          ok: 0,
          pending: 0,
          errors: 0,
          error: "Cannot connect to Rift database"
        }
      }
    end
  end
end
