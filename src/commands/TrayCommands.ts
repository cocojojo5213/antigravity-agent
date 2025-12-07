import { invoke } from '@tauri-apps/api/core';
import type { TrayToggleResult } from './types/tray.types';

/**
 * 系统托盘命令
 */
export class TrayCommands {
  /**
   * 启用系统托盘
   * @returns 启用结果消息
   */
  static async enable(): Promise<string> {
    return invoke('enable_system_tray');
  }

  /**
   * 禁用系统托盘
   * @returns 禁用结果消息
   */
  static async disable(): Promise<string> {
    return invoke('disable_system_tray');
  }

  /**
   * 切换系统托盘状态
   * @returns 切换结果
   */
  static async toggle(): Promise<TrayToggleResult> {
    return invoke('toggle_system_tray');
  }

  /**
   * 获取系统托盘状态
   * @returns 是否已启用
   */
  static async isEnabled(): Promise<boolean> {
    return invoke('get_system_tray_state');
  }

  /**
   * 最小化窗口到托盘
   * @returns 最小化结果消息
   */
  static async minimize(): Promise<string> {
    return invoke('minimize_to_tray');
  }

  /**
   * 从托盘恢复窗口
   * @returns 恢复结果消息
   */
  static async restore(): Promise<string> {
    return invoke('restore_from_tray');
  }

  
  /**
   * 更新托盘菜单
   * @param accounts 账户邮箱列表
   * @returns 更新结果消息
   */
  static async updateMenu(accounts: string[]): Promise<string> {
    return invoke('update_tray_menu_command', { accounts });
  }
}
