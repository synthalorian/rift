require "test_helper"

class PipelineRunsControllerTest < ActionDispatch::IntegrationTest
  test "should get index" do
    get pipeline_runs_url
    assert_response :success
  end

  test "index has pipeline runs card" do
    get pipeline_runs_url
    assert_select ".card-title", text: "Pipeline Runs"
  end

  test "index renders without Rift DB" do
    get pipeline_runs_url
    assert_response :success
  end

  test "show with unknown run redirects" do
    get pipeline_run_url("nonexistent-run-id")
    assert_redirected_to pipeline_runs_path
  end

  test "show renders gracefully without Rift DB" do
    get pipeline_run_url("some-run-id")
    assert_response :redirect
  end

  test "index shows runs or instructions" do
    get pipeline_runs_url
    assert_response :success
    # Either there's a table with runs or a message to run rift
    assert_select ".card" do
      assert_select "table, p", minimum: 1
    end
  end
end
