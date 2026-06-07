import { invoke } from '@tauri-apps/api/core'

/**
 * Window Animation Composable
 * Handles smooth Tauri window resizing and positioning
 */

let isAnimating = false

export function useWindowAnimate(): {
  animateResize: (
    targetWidth: number,
    targetHeight: number,
    targetX: number,
    targetY: number,
    duration?: number
  ) => void
} {
  /**
   * Animates the window bounds (width, height, x, y) over a duration
   */
  const animateResize = (
    targetWidth: number,
    targetHeight: number,
    targetX: number,
    targetY: number,
    duration = 200
  ): void => {
    if (isAnimating) return
    isAnimating = true

    const startWidth = window.outerWidth
    const startHeight = window.outerHeight
    const startX = window.screenX
    const startY = window.screenY
    const startTime = performance.now()

    const animate = (currentTime: number): void => {
      const elapsed = currentTime - startTime
      const progress = Math.min(elapsed / duration, 1)

      // Use easeOutQuart for smooth stopping
      const easeProgress = 1 - Math.pow(1 - progress, 4)

      const currentWidth = Math.floor(startWidth + (targetWidth - startWidth) * easeProgress)
      const currentHeight = Math.floor(startHeight + (targetHeight - startHeight) * easeProgress)
      const currentX = Math.floor(startX + (targetX - startX) * easeProgress)
      const currentY = Math.floor(startY + (targetY - startY) * easeProgress)

      invoke('resize_window', { width: currentWidth, height: currentHeight, x: currentX, y: currentY })

      if (progress < 1) {
        requestAnimationFrame(animate)
      } else {
        isAnimating = false
      }
    }

    requestAnimationFrame(animate)
  }

  return {
    animateResize
  }
}
