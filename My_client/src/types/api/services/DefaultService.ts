/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
 
import type { Alarm } from '../models/Alarm';
import type { Driver } from '../models/Driver';
import type { Order } from '../models/Order';
import type { Pagination } from '../models/Pagination';
import type { Vehicle } from '../models/Vehicle';
import type { CancelablePromise } from '../core/CancelablePromise';
import { OpenAPI } from '../core/OpenAPI';
import { request as __request } from '../core/request';
export class DefaultService {
    /**
     * 获取车辆列表
     * @param page
     * @param pageSize
     * @returns any 成功
     * @throws ApiError
     */
    public static getApiVehicles(
        page?: number,
        pageSize?: number,
    ): CancelablePromise<{
        success?: boolean;
        data?: Array<Vehicle>;
        pagination?: Pagination;
    }> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/vehicles',
            query: {
                'page': page,
                'page_size': pageSize,
            },
        });
    }
    /**
     * 获取订单列表
     * @param page
     * @param pageSize
     * @returns any 成功
     * @throws ApiError
     */
    public static getApiOrders(
        page?: number,
        pageSize?: number,
    ): CancelablePromise<{
        success?: boolean;
        data?: Array<Order>;
        pagination?: Pagination;
    }> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/orders',
            query: {
                'page': page,
                'page_size': pageSize,
            },
        });
    }
    /**
     * 获取司机列表
     * @param page
     * @param pageSize
     * @returns any 成功
     * @throws ApiError
     */
    public static getApiDrivers(
        page?: number,
        pageSize?: number,
    ): CancelablePromise<{
        success?: boolean;
        data?: Array<Driver>;
        pagination?: Pagination;
    }> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/drivers',
            query: {
                'page': page,
                'page_size': pageSize,
            },
        });
    }
    /**
     * 获取报警列表
     * @param page
     * @param pageSize
     * @returns any 成功
     * @throws ApiError
     */
    public static getApiAlerts(
        page?: number,
        pageSize?: number,
    ): CancelablePromise<{
        success?: boolean;
        data?: Array<Alarm>;
        pagination?: Pagination;
    }> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/api/alerts',
            query: {
                'page': page,
                'page_size': pageSize,
            },
        });
    }
}


