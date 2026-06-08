import { invoke } from '@tauri-apps/api/core'

/**
 * Window Animation Composable
 * Handles smooth Tauri window resizing and positioning
 */

let animationId: number | null = null

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
   * Animates the window bounds with throttled IPC calls
   */
  const animateResize = (
    targetWidth: number,
    targetHeight: number,
    targetX: number,
    targetY: number,
    duration = 150
  ): void => {
    // 如果正在动画，取消当前动画
    if (animationId) {
      cancelAnimationFrame(animationId)
      animationId = null
    }

    const startWidth = window.outerWidth
    const startHeight = window.outerHeight
    const startX = window.screenX
    const startY = window.screenY
    const startTime = performance.now()

    // 每 50ms 更新一次窗口位置（约 20fps）
    const updateInterval = 50
    let lastUpdateTime = 0

    const animate = (currentTime: number): void => {
      const elapsed = currentTime - startTime
      const progress = Math.min(elapsed / duration, 1)

      // 使用 easeOutQuart 缓动函数
      const easeProgress = 1 - Math.pow(1 - progress, 4)

      const currentWidth = Math.floor(startWidth + (targetWidth - startWidth) * easeProgress)
      const currentHeight = Math.floor(startHeight + (targetHeight - startHeight) * easeProgress)
      const currentX = Math.floor(startX + (targetX - startX) * easeProgress)
      const currentY = Math.floor(startY + (targetY - startY) * easeProgress)

      // 节流：只在间隔时间到达或动画结束时更新窗口
      if (currentTime - lastUpdateTime >= updateInterval || progress >= 1) {
        lastUpdateTime = currentTime
        invoke('resize_window', { 
          width: currentWidth, 
          height: currentHeight, 
          x: currentX, 
          y: currentY 
        })
      }

      if (progress < 1) {
        animationId = requestAnimationFrame(animate)
      } else {
        animationId = null
      }
    }

    animationId = requestAnimationFrame(animate)
  }

  return {
    animateResize
  }
}
