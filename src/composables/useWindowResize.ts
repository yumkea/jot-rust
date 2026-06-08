import { getCurrentWindow } from '@tauri-apps/api/window'

/**
 * 窗口缩放逻辑 Composables
 */
export function useWindowResize(): {
  startResize: (direction: string, e: MouseEvent) => void
} {
  const appWindow = getCurrentWindow()
  type ResizeDirection = Parameters<typeof appWindow.startResizeDragging>[0]

  /**
   * 全方位窗口缩放逻辑
   * @param direction 缩放方向：'n'(北), 's'(南), 'e'(东), 'w'(西) 及其组合
   */
  const startResize = (direction: string, e: MouseEvent): void => {
    e.preventDefault()
    e.stopPropagation()

    void appWindow.startResizeDragging(direction as ResizeDirection)
  }

  return {
    startResize
  }
}
