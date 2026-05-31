require "test_helper"

class AssetsControllerTest < ActionDispatch::IntegrationTest
  test "should get index" do
    get assets_url
    assert_response :success
  end

  test "index has asset browser card" do
    get assets_url
    assert_select ".card-title", text: "Asset Browser"
  end

  test "index renders gracefully without Rift DB" do
    get assets_url
    assert_response :success
  end

  test "index supports status filter" do
    get assets_url, params: { status: "ok" }
    assert_response :success
  end

  test "index supports search query" do
    get assets_url, params: { q: "test" }
    assert_response :success
  end

  test "show returns 404 for missing asset" do
    get asset_url("nonexistent/file.png")
    assert_redirected_to assets_path
  end

  test "show renders gracefully without Rift DB" do
    get asset_url("some%2Fpath%2Fasset.png")
    assert_response :redirect
  end

  test "index has status filter links" do
    get assets_url
    assert_select ".filter-bar a", 4
    assert_select 'a[href="/assets?status=ok"]', text: /✓ OK/
    assert_select 'a[href="/assets?status=error"]', text: /✗ Errors/
  end
end
