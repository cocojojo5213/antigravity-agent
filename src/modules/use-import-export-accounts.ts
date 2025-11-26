/**
 * 配置管理 Store (完全集成版)
 * 直接使用 Zustand，集成所有配置管理逻辑，提供完整接口
 */

import {create} from 'zustand';
import {open, save} from '@tauri-apps/plugin-dialog';
import {invoke} from '@tauri-apps/api/core';
import {logger} from '@/utils/logger.ts';
import toast from 'react-hot-toast';
import {BackupCommands} from "@/commands/BackupCommands.ts";
import {BackupData} from "@/commands/types/backup.types.ts";
import {LoggingCommands} from "@/commands/LoggingCommands.ts";

interface EncryptedConfigData {
  version: string;
  backupCount: number;
  backups: BackupData[];
}

// Store 状态
interface ConfigState {
  isImporting: boolean;
  isExporting: boolean;
  isCheckingData: boolean;
  // 对话框状态
  importDialogIsOpen: boolean;
  exportDialogIsOpen: boolean;
  // 待处理操作数据
  pendingImportPath?: string;
  pendingExportData?: BackupData[];
}

// Store 操作
interface ConfigActions {
  setImporting: (isImporting: boolean) => void;
  setExporting: (isExporting: boolean) => void;
  setCheckingData: (isCheckingData: boolean) => void;
  // 对话框控制
  openImportDialog: (filePath: string) => void;
  closeImportDialog: () => void;
  openExportDialog: (backupData: BackupData[]) => void;
  closeExportDialog: () => void;
  // 密码提交处理
  submitImportPassword: (password: string) => Promise<void>;
  submitExportPassword: (password: string) => Promise<void>;
  // 主要操作
  importConfig: () => Promise<void>;
  exportConfig: () => Promise<void>;
}

// 创建 Zustand Store
export const useImportExportAccount = create<ConfigState & ConfigActions>()(
  (set, get) => {
    return {
      // 初始状态
      isImporting: false,
      isExporting: false,
      isCheckingData: false,
      // 对话框初始状态
      importDialogIsOpen: false,
      exportDialogIsOpen: false,
      // 待处理操作数据
      pendingImportPath: undefined,
      pendingExportData: undefined,

      // 状态设置方法
      setImporting: (isImporting: boolean) => set({ isImporting }),
      setExporting: (isExporting: boolean) => set({ isExporting }),
      setCheckingData: (isCheckingData: boolean) => set({ isCheckingData }),

      // 打开导入对话框
      openImportDialog: (filePath: string) => set({
        importDialogIsOpen: true,
        pendingImportPath: filePath
      }),

      // 关闭导入对话框
      closeImportDialog: () => set({
        importDialogIsOpen: false,
        pendingImportPath: undefined
      }),

      // 打开导出对话框
      openExportDialog: (backupData: BackupData[]) => set({
        exportDialogIsOpen: true,
        pendingExportData: backupData
      }),

      // 关闭导出对话框
      closeExportDialog: () => set({
        exportDialogIsOpen: false,
        pendingExportData: undefined
      }),

      // ============ 密码提交处理 ============
      submitImportPassword: async (password: string): Promise<void> => {
        // 在方法开始时捕获所需状态，避免竞态条件
        const { pendingImportPath } = get();
        if (!pendingImportPath) {
          toast.error('没有待处理的导入文件');
          return;
        }

        try {
          get().closeImportDialog();
          set({ isImporting: true });
          toast.loading('正在使用 Rust 解密文件...', {duration: 1});

          // 调用后端解密
          const decryptedJson: string = await invoke('decrypt_config_data', {
            filePath: pendingImportPath,
            password
          });
          const configData: EncryptedConfigData = JSON.parse(decryptedJson);

          // 验证配置数据格式
          if (!configData.version || !configData.backups || !Array.isArray(configData.backups)) {
            throw new Error('配置文件格式无效');
          }

          logger.info('开始恢复备份数据', {
            module: 'useImportExportAccount',
            backupCount: configData.backups.length
          });
          toast.loading('正在恢复账户数据...', {duration: 1});

          const result = await BackupCommands.restoreFiles(configData.backups);

          if (result.failed.length > 0) {
            logger.warn('部分文件恢复失败', {
              module: 'useImportExportAccount',
              restoredCount: result.restoredCount,
              failedCount: result.failed.length,
              failedFiles: result.failed
            });
            toast.success(`配置文件导入成功，已恢复 ${result.restoredCount} 个账户，${result.failed.length} 个失败`);
          } else {
            logger.info('所有文件恢复成功', {
              module: 'useImportExportAccount',
              restoredCount: result.restoredCount
            });
            toast.success(`配置文件导入成功，已恢复 ${result.restoredCount} 个账户`);
          }
        } catch (error) {
          logger.error('导入失败', {
            module: 'useImportExportAccount',
            stage: 'import_process',
            error: error instanceof Error ? error.message : String(error)
          });
          toast.error(`配置文件导入失败: ${error instanceof Error ? error.message : String(error)}`);
        } finally {
          set({ isImporting: false });
        }
      },

      submitExportPassword: async (password: string): Promise<void> => {
        // 在方法开始时捕获所需状态，避免竞态条件
        const { pendingExportData } = get();
        if (!pendingExportData || pendingExportData.length === 0) {
          toast.error('没有待处理的导出数据');
          return;
        }

        try {
          get().closeExportDialog();
          set({ isExporting: true });
          toast.loading('正在生成加密配置文件...', {duration: 1});

          // 构建配置数据
          const configData: EncryptedConfigData = {
            version: '1.1.0',
            backupCount: pendingExportData.length,
            backups: pendingExportData
          };

          // 调用后端加密
          const configJson = JSON.stringify(configData, null, 2);
          const configSize = new Blob([configJson]).size;

          logger.info('配置数据已生成', {
            module: 'useImportExportAccount',
            backupCount: pendingExportData.length,
            configSize
          });

          const encryptedData = await invoke<string>('encrypt_config_data', {
            jsonData: configJson,
            password
          });

          // 选择保存位置
          const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-');
          const defaultFileName = `antigravity_encrypted_config_${timestamp}.enc`;

          const savePath = await save({
            title: '保存配置文件',
            defaultPath: defaultFileName,
            filters: [
              {
                name: 'Antigravity 加密配置文件',
                extensions: ['enc']
              }
            ]
          });

          if (!savePath || typeof savePath !== 'string') {
            logger.warn('未选择保存位置', {
              module: 'useImportExportAccount'
            });
            toast.error('未选择保存位置');
            return;
          }

          // 保存加密文件
          await LoggingCommands.writeTextFile(savePath, encryptedData);

          toast.success(`配置文件已保存: ${savePath}`);
          logger.info('导出配置成功', {
            module: 'useImportExportAccount',
            savePath,
            backupCount: pendingExportData.length,
            configSize
          });

        } catch (error) {
          logger.error('导出失败', {
            module: 'useImportExportAccount',
            stage: 'password_validation',
            error: error instanceof Error ? error.message : String(error)
          });
          toast.error(`导出配置文件失败: ${error instanceof Error ? error.message : String(error)}`);
        } finally {
          set({ isExporting: false });
        }
      },

      // ============ 导入配置 ============
      importConfig: async (): Promise<void> => {
        logger.info('开始导入配置文件', { module: 'useImportExportAccount' });

        try {
          // 选择文件
          const selected = await open({
            title: '选择配置文件',
            filters: [
              {
                name: '加密配置文件',
                extensions: ['enc']
              },
              {
                name: '所有文件',
                extensions: ['*']
              }
            ],
            multiple: false
          });

          if (!selected || typeof selected !== 'string') {
            logger.warn('未选择文件', {
              module: 'useImportExportAccount'
            });
            toast.error('未选择文件');
            return;
          }

          logger.info('已选择文件', {
            module: 'useImportExportAccount',
            filePath: selected
          });

          // 显示密码对话框，存储文件路径
          get().openImportDialog(selected);

        } catch (error) {
          logger.error('文件操作失败', {
            module: 'useImportExportAccount',
            stage: 'file_selection',
            error: error instanceof Error ? error.message : String(error)
          });
          toast.error(`文件操作失败: ${error instanceof Error ? error.message : String(error)}`);
        }
      },

      // ============ 导出配置 ============
      exportConfig: async (): Promise<void> => {
        logger.info('开始导出配置', { module: 'useImportExportAccount' });

        try {
          toast.loading('正在收集账户数据...', {duration: 1});

          // ✅ 获取包含完整内容的备份数据
          const backupsWithContent = await invoke<BackupData[]>('collect_backup_contents');

          if (backupsWithContent.length === 0) {
            logger.warn('没有找到用户信息', {
              module: 'useImportExportAccount'
            });
            toast.error('没有找到任何用户信息，无法导出配置文件');
            return;
          }

          logger.info('找到备份数据', {
            module: 'useImportExportAccount',
            backupCount: backupsWithContent.length
          });

          
          // 显示密码对话框，传递备份数据
          get().openExportDialog(backupsWithContent);

        } catch (error) {
          logger.error('检查数据失败', {
            module: 'useImportExportAccount',
            stage: 'data_collection',
            error: error instanceof Error ? error.message : String(error)
          });
          toast.error(`检查数据失败: ${error instanceof Error ? error.message : String(error)}`);
        }
      }
    };
  }
);
