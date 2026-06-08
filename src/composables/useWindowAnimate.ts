import { invoke } from '@tauri-apps/api/core'

/**
 * Window Animation Composable
 * Handles smooth Tauri window resizing and positioning
 */

let animationId: number | null = null
let animationVersion = 0

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
   * Animates the window bounds with optimized performance
   */
  const animateResize = (
    targetWidth: number,
    targetHeight: number,
    targetX: number,
    targetY: number,
    _duration = 0
  ): void => {
    if (animationId) {
      cancelAnimationFrame(animationId)
      animationId = null
    }

    animationVersion += 1

    void invoke('resize_window', {
      width: Math.round(targetWidth),
      height: Math.round(targetHeight),
      x: Math.round(targetX),
      y: Math.round(targetY)
    }).catch((error) => console.error('Failed to resize window:', error))
  }

  return {
    animateResize
  }
}
