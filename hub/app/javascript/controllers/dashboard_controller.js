import { Controller } from "@hotwired/stimulus"

// Connects to data-controller="dashboard"
export default class extends Controller {
  static targets = ["stats", "lastRun", "recentRuns", "indicator", "timestamp"]
  static values = { refreshInterval: { type: Number, default: 15 } }

  connect() {
    if (this.hasStatsTarget) {
      this.startAutoRefresh()
    }
  }

  disconnect() {
    this.stopAutoRefresh()
  }

  startAutoRefresh() {
    // Initial load
    this.refresh()

    // Set up periodic refresh
    this.intervalId = setInterval(() => {
      this.refresh()
    }, this.refreshIntervalValue * 1000)
  }

  stopAutoRefresh() {
    if (this.intervalId) {
      clearInterval(this.intervalId)
      this.intervalId = null
    }
  }

  async refresh() {
    try {
      const response = await fetch("/?format=json")
      if (!response.ok) return

      const data = await response.json()

      // Update stat cards via targets
      if (this.hasStatsTarget) {
        this.statsTargets.forEach(card => {
          const metric = card.dataset.metric
          const valueEl = card.querySelector(".stat-value")
          if (valueEl && data[metric] !== undefined) {
            // Animate the number change
            const oldVal = parseInt(valueEl.textContent) || 0
            const newVal = data[metric]
            if (oldVal !== newVal) {
              card.classList.add("updated")
              setTimeout(() => card.classList.remove("updated"), 1000)
            }
            valueEl.textContent = newVal
          }
        })
      }

      // Update timestamp
      if (this.hasTimestampTarget) {
        this.timestampTarget.textContent = new Date().toISOString()
      }

      // Pulse the live indicator
      if (this.hasIndicatorTarget) {
        this.indicatorTarget.classList.add("pulse")
        setTimeout(() => this.indicatorTarget.classList.remove("pulse"), 300)
      }

    } catch (e) {
      // Silently fail — will retry on next interval
    }
  }
}
