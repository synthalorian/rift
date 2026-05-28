class AssetsController < ApplicationController
  def index
    @title = "Assets"
    @status_filter = params[:status]
    @search_query = params[:q]
    if @search_query.present?
      @assets = RiftDb.search_assets(@search_query, status: @status_filter.presence, limit: 100)
    else
      @assets = RiftDb.assets(status: @status_filter.presence, limit: 100, offset: 0)
    end
    @counts = RiftDb.asset_counts
  rescue SQLite3::Exception => e
    @error = "Cannot read Rift database: #{e.message}"
    @assets = []
    @counts = {"ok" => 0, "pending" => 0, "error" => 0}
  end

  def show
    @asset = RiftDb.find_asset(params[:id])
    if @asset.nil?
      redirect_to assets_path, alert: "Asset not found"
      return
    end
    @title = @asset["relative_path"]
  rescue SQLite3::Exception => e
    @error = "Cannot read Rift database: #{e.message}"
  end
end
