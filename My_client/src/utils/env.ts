export const isRemoteOpsEnabled = (): boolean => {
  return import.meta.env.VITE_ENABLE_REMOTE_OPS === 'true';
};

export const getProductType = (): string => {
  return import.meta.env.VITE_PRODUCT_TYPE || 'carptms';
};

export const isProduction = (): boolean => {
  return import.meta.env.PROD;
};