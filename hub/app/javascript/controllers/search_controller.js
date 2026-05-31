import { Controller } from "@hotwired/stimulus"

// Connects to data-controller="search"
export default class extends Controller {
  static targets = ["input", "results", "statusCount"]
  static values = {
    url: String,
    minLength: { type: Number, default: 2 },
    delay: { type: Number, default: 300 }
  }

  connect() {
    this.timeout = null
  }

  disconnect() {
    if (this.timeout) {
      clearTimeout(this.timeout)
    }
  }

  search() {
    if (this.timeout) {
      clearTimeout(this.timeout)
    }

    const query = this.inputTarget.value.trim()

    if (query.length < this.minLengthValue) {
      // Reset to full asset list
      if (query.length === 0) {
        this.loadResults("")
      }
      return
    }

    this.timeout = setTimeout(() => {
      this.loadResults(query)
    }, this.delayValue)
  }

  async loadResults(query) {
    if (!this.hasResultsTarget) return

    try {
      const params = new URLSearchParams()
      if (query) params.set("q", query)

      const response = await fetch(`${this.urlValue}?${params.toString()}`, {
        headers: { "Accept": "text/vnd.turbo-stream.html" }
      })

      if (response.ok) {
        const html = await response.text()
        // Replace the table body with Turbo stream results
        this.resultsTarget.innerHTML = html
      }
    } catch (e) {
      // Silently fail
    }
  }

  clear() {
    this.inputTarget.value = ""
    this.search()
  }
}
