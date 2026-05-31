import { Controller } from "@hotwired/stimulus"

// Connects to data-controller="filter"
export default class extends Controller {
  static targets = ["link"]

  connect() {
    // Highlight active filter based on current URL
    const current = window.location.search
    this.linkTargets.forEach(link => {
      const href = link.getAttribute("href") || ""
      const linkParams = href.includes("?") ? href.substring(href.indexOf("?")) : ""
      if (linkParams === current || (current === "" && linkParams === "")) {
        link.classList.add("active")
      } else {
        link.classList.remove("active")
      }
    })
  }

  select(event) {
    event.preventDefault()
    const link = event.currentTarget

    // Update active state
    this.linkTargets.forEach(l => l.classList.remove("active"))
    link.classList.add("active")

    // Navigate
    const href = link.getAttribute("href")
    if (href) {
      Turbo.visit(href)
    }
  }
}
