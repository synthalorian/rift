class AssetErrorsController < ApplicationController
  def index
    @title = "Errors"
    @errors = RiftDb.asset_errors(limit: 100)
  rescue SQLite3::Exception => e
    @error = "Cannot read Rift database: #{e.message}"
    @errors = []
  end
end
