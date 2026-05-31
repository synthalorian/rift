require "test_helper"

class AssetErrorsControllerTest < ActionDispatch::IntegrationTest
  test "should get index" do
    get errors_url
    assert_response :success
  end

  test "index has errors card" do
    get errors_url
    assert_select ".card-title", text: "Asset Errors"
  end

  test "index renders without Rift DB" do
    get errors_url
    assert_response :success
  end

  test "index shows clean state when no errors" do
    get errors_url
    assert_select "p", /No errors/
    assert_select "p", /Pipeline is clean/
  end
end
