require "test_helper"

class DashboardControllerTest < ActionDispatch::IntegrationTest
  test "should get index" do
    get root_url
    assert_response :success
    assert_select "title", /Rift/
  end

  test "index includes stat cards" do
    get root_url
    assert_select ".stat-card", 4
    assert_select ".stat-label", text: "Total Assets"
    assert_select ".stat-label", text: "Converted ✓"
    assert_select ".stat-label", text: "Pending ⋯"
    assert_select ".stat-label", text: "Errors ✗"
  end

  test "index shows no-database error state gracefully" do
    get root_url
    # Should render without crashing even without Rift database
    assert_response :success
  end

  test "index has navigation links" do
    get root_url
    assert_select "nav a", 4
    assert_select "nav a", text: "Dashboard"
    assert_select "nav a", text: "Assets"
    assert_select "nav a", text: "Runs"
    assert_select "nav a", text: "Errors"
  end

  test "index responds to json format" do
    get root_url(format: :json)
    assert_response :success
    body = JSON.parse(@response.body)
    assert_includes body, "total_assets"
    assert_includes body, "counts"
    assert_includes body, "last_run"
  end
end
