import { invoke } from '@tauri-apps/api/core'

/**
 * Window Animation Composable
 * Handles smooth Tauri window resizing and positioning
 */

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
   * Resizes the window to target bounds immediately (no animation for performance)
   */
  const animateResize = (
    targetWidth: number,
    targetHeight: number,
    targetX: number,
    targetY: number,
    _duration = 200
  ): void => {
    // 直接设置窗口大小，不做动画以提高性能
    invoke('resize_window', { 
      width: targetWidth, 
      height: targetHeight, 
      x: targetX, 
      y: targetY 
    })
  }

  return {
    animateResize
  }
}
