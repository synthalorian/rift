Rails.application.routes.draw do
  root "dashboard#index"

  resources :pipeline_runs, only: [:index, :show], path: "runs"
  resources :errors, only: [:index], controller: "asset_errors"

  # Assets with glob route for paths containing dots (e.g. icon.png, assets/bg.psd)
  get "assets" => "assets#index", as: :assets
  get "assets/*id" => "assets#show", as: :asset, format: false
end
