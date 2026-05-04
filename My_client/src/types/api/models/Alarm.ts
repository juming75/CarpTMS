/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
 
export type Alarm = {
    id?: number;
    vehicle_id?: number;
    plate_number?: string;
    alarm_type?: string;
    alarm_level?: string;
    alarm_time?: string;
    location?: {
        latitude?: number;
        longitude?: number;
    };
    status?: string;
    handled?: boolean;
};



