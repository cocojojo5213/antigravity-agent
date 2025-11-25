import { invoke } from '@tauri-apps/api/core';
import type { AppSettings } from './types/settings.types';

/**
 * 设置管理命令
 */
export class SettingsCommands {

  /**
   * 获取静默启动状态
   * @returns 是否已启用静默启动
   */
  static async isSilentStartEnabled(): Promise<boolean> {
    return invoke('is_silent_start_enabled');
  }

  /**
   * 保存静默启动状态
   * @param enabled 是否启用
   * @returns 保存结果消息
   */
  static async saveSilentStartState(enabled: boolean): Promise<string> {
    return invoke('save_silent_start_state', { enabled });
  }

  /**
   * 获取所有应用设置
   * @returns 应用设置对象
   */
  static async getAll(): Promise<AppSettings> {
    return invoke('get_all_settings');
  }
}
