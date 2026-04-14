/**
 * 协议适配器模块
 * 统一导出协议相关的所有类型和接口
 */

// 旧协议适配器
export {
  LegacyProtocolAdapter,
  legacyProtocolAdapter,
  createLegacyProtocolAdapter,
  type LegacyCommand,
  type LegacyRequest,
  type LegacyResponse,
  type LoginRequestData,
  type LoginResponseData,
  type VehicleRequestData,
  type VehicleResponseData,
} from './legacyProtocolAdapter';

// 新协议适配器
export {
  NewProtocolAdapter,
  newProtocolAdapter,
  createNewProtocolAdapter,
  type NewAction,
  type NewResource,
  type NewRequest,
  type NewResponse,
} from './newProtocolAdapter';

// 协议管理器
export {
  ProtocolManager,
  protocolManager,
  createProtocolManager,
  type ProtocolVersion,
  type IProtocolAdapter,
  type ProtocolManagerConfig,
} from './protocolManager';


