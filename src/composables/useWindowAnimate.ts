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
  ) => Promise<void>
} {
  /**
   * Applies native window bounds once. Visual motion stays inside the WebView
   * with transform/opacity so the window manager does not re-layout every frame.
   */
  const animateResize = (
    targetWidth: number,
    targetHeight: number,
    targetX: number,
    targetY: number,
    _duration = 0
  ): Promise<void> => {
    return invoke<void>('resize_window', {
      width: Math.round(targetWidth),
      height: Math.round(targetHeight),
      x: Math.round(targetX),
      y: Math.round(targetY)
    }).catch((error) => {
      console.error('Failed to resize window:', error)
    })
  }

  return {
    animateResize
  }
}
