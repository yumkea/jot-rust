import { getCurrentWindow } from '@tauri-apps/api/window'

/**
 * 窗口缩放逻辑 Composables
 */
export function useWindowResize(): {
  startResize: (direction: string, e: MouseEvent) => void
} {
  type AppWindow = ReturnType<typeof getCurrentWindow>
  type ResizeDirection = Parameters<AppWindow['startResizeDragging']>[0]
  const directionMap: Record<string, ResizeDirection> = {
    n: 'North',
    s: 'South',
    e: 'East',
    w: 'West',
    nw: 'NorthWest',
    ne: 'NorthEast',
    sw: 'SouthWest',
    se: 'SouthEast'
  }

  /**
   * 全方位窗口缩放逻辑
   * @param direction 缩放方向：'n'(北), 's'(南), 'e'(东), 'w'(西) 及其组合
   */
  const startResize = (direction: string, e: MouseEvent): void => {
    e.preventDefault()
    e.stopPropagation()

    const resizeDirection = directionMap[direction]
    if (!resizeDirection) return
    if (!('__TAURI_INTERNALS__' in window)) return

    try {
      void getCurrentWindow().startResizeDragging(resizeDirection).catch((error) => {
        console.error('Failed to start resize dragging:', error)
      })
    } catch (error) {
      console.error('Failed to start resize dragging:', error)
    }
  }

  return {
    startResize
  }
}
