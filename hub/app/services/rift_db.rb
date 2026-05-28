# Service for reading the Rift SQLite database
class RiftDb
  class << self
    def db_path
      @db_path ||= find_db_path
    end

    def db_path=(path)
      @db_path = path
      @connection = nil
    end

    def connected?
      return false unless db_path && File.exist?(db_path)
      connection
      true
    rescue SQLite3::Exception
      false
    end

    def asset_counts
      return default_counts unless connected?
      rows = query("SELECT status, COUNT(*) as count FROM assets GROUP BY status")
      counts = default_counts
      rows.each { |status, count| counts[status] = count if status }
      counts
    end

    def total_assets
      return 0 unless connected?
      row = query_one("SELECT COUNT(*) as total FROM assets")
      row ? row[0] : 0
    end

    def recent_runs(limit: 10)
      return [] unless connected?
      query("SELECT id, timestamp, status, converted, errors, summary FROM pipeline_runs ORDER BY timestamp DESC LIMIT ?", limit).map do |row|
        {
          "id" => row[0],
          "timestamp" => row[1],
          "status" => row[2],
          "converted" => row[3],
          "errors" => row[4],
          "summary" => row[5]
        }
      end
    end

    def assets(status: nil, limit: 50, offset: 0)
      return [] unless connected?
      rows = if status
        query("SELECT relative_path, sha256, last_modified, last_converted, status, error_message FROM assets WHERE status = ? ORDER BY last_modified DESC LIMIT ? OFFSET ?", status, limit, offset)
      else
        query("SELECT relative_path, sha256, last_modified, last_converted, status, error_message FROM assets ORDER BY last_modified DESC LIMIT ? OFFSET ?", limit, offset)
      end
      rows.map do |row|
        {
          "relative_path" => row[0],
          "sha256" => row[1],
          "last_modified" => row[2],
          "last_converted" => row[3],
          "status" => row[4],
          "error_message" => row[5]
        }
      end
    end

    def search_assets(query, status: nil, limit: 50)
      return [] unless connected?
      like = "%#{query}%"
      if status
        rows = query("SELECT relative_path, sha256, last_modified, last_converted, status, error_message FROM assets WHERE relative_path LIKE ? AND status = ? ORDER BY last_modified DESC LIMIT ?", like, status, limit)
      else
        rows = query("SELECT relative_path, sha256, last_modified, last_converted, status, error_message FROM assets WHERE relative_path LIKE ? ORDER BY last_modified DESC LIMIT ?", like, limit)
      end
      rows.map do |row|
        {
          "relative_path" => row[0],
          "sha256" => row[1],
          "last_modified" => row[2],
          "last_converted" => row[3],
          "status" => row[4],
          "error_message" => row[5]
        }
      end
    end

    def asset_errors(limit: 50)
      return [] unless connected?
      query(
        "SELECT ae.id, ae.run_id, ae.relative_path, ae.error_type, ae.message, ae.timestamp, a.relative_path
         FROM asset_errors ae LEFT JOIN assets a ON ae.relative_path = a.relative_path
         ORDER BY ae.timestamp DESC LIMIT ?", limit
      ).map do |row|
        {
          "id" => row[0],
          "run_id" => row[1],
          "relative_path" => row[2],
          "error_type" => row[3],
          "message" => row[4],
          "timestamp" => row[5]
        }
      end
    end

    def last_run
      return nil unless connected?
      row = query_one("SELECT * FROM pipeline_runs ORDER BY timestamp DESC LIMIT 1")
      return nil unless row
      {
        "id" => row[0],
        "timestamp" => row[1],
        "status" => row[2],
        "total_assets" => row[3],
        "converted" => row[4],
        "errors" => row[5],
        "summary" => row[6]
      }
    end

    def find_asset(relative_path)
      return nil unless connected?
      row = query_one("SELECT relative_path, sha256, last_modified, last_converted, status, error_message FROM assets WHERE relative_path = ?", relative_path)
      return nil unless row
      {
        "relative_path" => row[0],
        "sha256" => row[1],
        "last_modified" => row[2],
        "last_converted" => row[3],
        "status" => row[4],
        "error_message" => row[5]
      }
    end

    def find_run(run_id)
      return nil unless connected?
      row = query_one("SELECT id, timestamp, status, total_assets, converted, errors, summary FROM pipeline_runs WHERE id = ?", run_id)
      return nil unless row
      {
        "id" => row[0],
        "timestamp" => row[1],
        "status" => row[2],
        "total_assets" => row[3],
        "converted" => row[4],
        "errors" => row[5],
        "summary" => row[6]
      }
    end

    def errors_for_run(run_id, limit: 50)
      return [] unless connected?
      query("SELECT id, relative_path, error_type, message, timestamp FROM asset_errors WHERE run_id = ? ORDER BY timestamp DESC LIMIT ?", run_id, limit).map do |row|
        {
          "id" => row[0],
          "relative_path" => row[1],
          "error_type" => row[2],
          "message" => row[3],
          "timestamp" => row[4]
        }
      end
    end

    private

    def connection
      @connection ||= SQLite3::Database.new(db_path)
    end

    def query(sql, *params)
      conn = connection
      conn.execute(sql, params)
    end

    def query_one(sql, *params)
      conn = connection
      conn.execute(sql, params).first
    end

    def default_counts
      {"ok" => 0, "pending" => 0, "error" => 0}
    end

    def find_db_path
      return ENV["RIFT_DB_PATH"] if ENV["RIFT_DB_PATH"] && File.exist?(ENV["RIFT_DB_PATH"])

      dir = Rails.root
      5.times do
        candidate = dir.join(".rift", "state.db")
        return candidate.to_s if File.exist?(candidate)
        dir = dir.parent
        break if dir.root?
      end

      File.join(Rails.root, ".rift", "state.db")
    end
  end
end
