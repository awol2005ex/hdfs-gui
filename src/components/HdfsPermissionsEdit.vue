<template>
    
   <table>
    <tr>
        <td>user</td>
        <td><input type="checkbox" v-model="permissionJson.user.read" />read</td>
        <td><input type="checkbox" v-model="permissionJson.user.write" />write</td>
        <td><input type="checkbox" v-model="permissionJson.user.execute" />execute</td>
    </tr>
    <tr>
        <td>group</td>
        <td><input type="checkbox" v-model="permissionJson.group.read" />read</td>
        <td><input type="checkbox" v-model="permissionJson.group.write" />write</td>
        <td><input type="checkbox" v-model="permissionJson.group.execute" />execute</td>
    </tr>
    <tr>
        <td>other</td>
        <td><input type="checkbox" v-model="permissionJson.other.read" />read</td>
        <td><input type="checkbox" v-model="permissionJson.other.write" />write</td>
        <td><input type="checkbox" v-model="permissionJson.other.execute" />execute</td>
    </tr>

    <tr>
        <td><input type="checkbox" v-model="recursive" />recursive</td>
        <td></td>
        <td></td>
        <td></td>
    </tr>
   </table>

</template>
<script setup lang="ts">
import { ref } from "vue";
interface Props {
  permission?: number;
}

const props = withDefaults(defineProps<Props>(), {
    permission: 509
});

interface PermissionSet {
    read: boolean,
    write: boolean,
    execute: boolean,
}
interface PermissionResult {
    user: PermissionSet,
    group: PermissionSet,
    other: PermissionSet,
}

const permissionJson = ref<PermissionResult>({
    user: {
        read: false,
        write: false,
        execute: false,
    },
    group: {
        read: false,
        write: false,
        execute: false,
    },
    other: {
        read: false,
        write: false,       
    execute: false,
    },
});

const recursive = ref(false);


const setPermissionsValue=(newPermission:number)=>{
    let octal = ("000" + newPermission.toString(8)).slice(-3);

    //console.log(octal)
    const p0 =parseInt(octal.charAt(0),10)
    //console.log(p0)
    const p1 =parseInt(octal.charAt(1),10)
    //console.log(p1)
    const p2 =parseInt(octal.charAt(2),10)
    //console.log(p2)
    //转成权限
    permissionJson.value = {
        user: {
            read:  (p0 & 4) >0,
            write: (p0 & 2) >0,
            execute: (p0 & 1) >0,
        },
        group: {
        
            read:  (p1 & 4) >0,
            write: (p1 & 2) >0,
            execute: (p1 & 1) >0,
        },
        other: {
            read:  (p2 & 4) >0,
            write: (p2 & 2) >0,
            execute: (p2 & 1) >0,
        },
    }
    //console.log(permissionJson)
}

const getNewPermission=()=>{
    let permission = [0,0,0];
    permission[0] += permissionJson.value.user.read ? 4 : 0;
    permission[0] += permissionJson.value.user.write ? 2 : 0;
    permission[0] += permissionJson.value.user.execute ? 1 : 0;
    permission[1] += permissionJson.value.group.read ? 4 : 0;
    permission[1] += permissionJson.value.group.write ? 2 : 0;
    permission[1] += permissionJson.value.group.execute ? 1 : 0;
    permission[2] += permissionJson.value.other.read ? 4 : 0;
    permission[2] += permissionJson.value.other.write ? 2 : 0;
    permission[2] += permissionJson.value.other.execute ? 1 : 0;
    return parseInt(permission.join(''),8);
}


defineExpose({
    setPermissionsValue,getNewPermission,recursive,props
});

</script>
<style scoped></style>    