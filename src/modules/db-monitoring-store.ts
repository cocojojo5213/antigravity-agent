/**
 * 数据库监控 Store - 简化版
 * 智能监控现在是默认功能，自动启动
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import {listen, UnlistenFn} from '@tauri-apps/api/event';
import { EventEmitter } from 'events';
import { logger } from '../utils/logger';

// 数据库变化事件数据接口
export interface DatabaseChangeEvent {
    timestamp: number;
    old_data?: any;
    new_data?: any;
    diff?: any;
    originalEvent?: any;
}

// 导出事件相关类型
export type { DatabaseEventMap, DatabaseEventListener };

// 全局数据库事件发射器
const databaseEventEmitter = new EventEmitter();

// 全局 unlistenFn 变量
let globalUnlistenFn: UnlistenFn | null = null;

// 数据库事件类型
export const DATABASE_EVENTS = {
  DATA_CHANGED: 'database:data-changed',
} as const;

// 事件类型映射
type DatabaseEventMap = {
  [DATABASE_EVENTS.DATA_CHANGED]: DatabaseChangeEvent;
};

// 事件监听器类型
type DatabaseEventListener<T extends keyof DatabaseEventMap> = (data: DatabaseEventMap[T]) => void;

// 操作接口 - 简化版，移除了设置管理
interface DbMonitoringActions {
  // 初始化监控（启动时调用）
  initializeMonitoring: () => Promise<void>;

  // 停止监听（清理资源）
  cleanup: () => Promise<void>;

  // 添加事件监听器
  addListener: <T extends keyof DatabaseEventMap>(
    event: T,
    listener: DatabaseEventListener<T>
  ) => (() => void);
}

// 创建 Store
export const useDbMonitoringStore = create<DbMonitoringActions>()(
  (set, get) => ({
      // 初始化监控（应用启动时调用）
      initializeMonitoring: async (): Promise<void> => {
        logger.info('初始化数据库监控', { module: 'DbMonitoringStore' });

        try {
          // 清理之前的监听器
          await get().cleanup();

          // 处理数据库变化事件
          const handleDatabaseChange = async (event: any) => {
            logger.info('接收到数据库变化事件', {
              module: 'DbMonitoringStore',
              eventId: event.id || 'unknown'
            });

            // 解析事件数据：newData, oldData, diff
            const { newData, oldData, diff } = event.payload;

            if (diff) {
              logger.info('数据库变化摘要', {
                module: 'DbMonitoringStore',
                hasChanges: diff.hasChanges,
                changedFieldsCount: diff.changedFields?.length || 0,
                summary: diff.summary
              });
            }

            // 发射内部数据库变化事件
            databaseEventEmitter.emit(DATABASE_EVENTS.DATA_CHANGED, {
              timestamp: Date.now(),
              newData,
              oldData,
              diff,
              originalEvent: event
            });

            logger.info('数据库变化事件已发射', {
              module: 'DbMonitoringStore'
            });
          };

          // 监听后端推送的数据库变化事件
          globalUnlistenFn = await listen('database-changed', handleDatabaseChange);

          // 启动后端监控
          await invoke('start_database_monitoring');

          logger.info('数据库监控已启动', {
            module: 'DbMonitoringStore'
          });
        } catch (error) {
          logger.error('启动数据库监控失败', {
            module: 'DbMonitoringStore',
            error: error instanceof Error ? error.message : String(error)
          });
        }
      },

      // 清理资源
      cleanup: async (): Promise<void> => {
        if (globalUnlistenFn) {
          try {
            await globalUnlistenFn();
            globalUnlistenFn = null;
            logger.info('数据库监听器已清理', {
              module: 'DbMonitoringStore'
            });
          } catch (error) {
            logger.warn('清理数据库监听器失败', {
              module: 'DbMonitoringStore',
              error: error instanceof Error ? error.message : String(error)
            });
          }
        }
      },

      addListener: <T extends keyof DatabaseEventMap>(
        event: T,
        listener: DatabaseEventListener<T>
      ): (() => void) => {
        databaseEventEmitter.on(event, listener);

        // 返回取消订阅函数
        return () => {
          databaseEventEmitter.off(event, listener);
        };
      },
    }),
);
