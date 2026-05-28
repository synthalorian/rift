class DashboardController < ApplicationController
  def index
    @title = "Rift Dashboard"
    @total_assets = RiftDb.total_assets
    @counts = RiftDb.asset_counts
    @recent_runs = RiftDb.recent_runs(limit: 5)
    @last_run = RiftDb.last_run
  rescue SQLite3::Exception => e
    @error = "Cannot connect to Rift database: #{e.message}"
    @total_assets = 0
    @counts = {"ok" => 0, "pending" => 0, "error" => 0}
    @recent_runs = []
    @last_run = nil
  end
end
